// Routes under the /colleges path

use actix_web::{get, web, HttpResponse};
use awc::Client;
use bb8_redis::{
    bb8::{self, PooledConnection},
    redis::{cmd, AsyncCommands},
    RedisConnectionManager,
};
use serde::{Deserialize, Serialize};

use crate::{app_state::AppState, structures::CollegeStruct};

const COLLEGE_LIST_EXP: usize = 24 * 60 * 60;

#[derive(Serialize)]
pub struct CollegeListResp {
    colleges: Option<Vec<CollegeStruct>>,
}

impl CollegeListResp {
    pub fn from(colleges: Vec<CollegeStruct>) -> Self {
        Self {
            colleges: Some(colleges),
        }
    }

    pub fn empty() -> Self {
        Self { colleges: None }
    }
}

#[get("/colleges/list-all")]
pub async fn hande_list_all_colleges(state: web::Data<AppState>) -> HttpResponse {
    match get_all_colleges(&state.redis_pool).await {
        Some(colleges) => HttpResponse::Ok().json(CollegeListResp::from(colleges)),
        None => HttpResponse::InternalServerError().json(CollegeListResp::empty()),
    }
}

#[derive(Deserialize)]
pub struct CollegeAPIResp {
    total_count: usize,
    results: Vec<CollegeStruct>,
}

async fn get_all_colleges(
    redis_pool: &bb8::Pool<RedisConnectionManager>,
) -> Option<Vec<CollegeStruct>> {
    // First, we will try to hit the cache.
    // If the cache misses due to any reason, we will default to retrieving the data from the API.
    if let Ok(mut redis_conn) = redis_pool.get().await {
        if let Ok(redis_cache_attempt) = cmd("GET")
            .arg("@COLLEGE_LIST/CACHE")
            .query_async::<_, Option<String>>(&mut *redis_conn)
            .await
        {
            // If the value is present, we can deserialize and return.
            if let Some(cached) = redis_cache_attempt {
                if let Ok(deserialized_cache) = serde_json::from_str::<Vec<CollegeStruct>>(&cached)
                {
                    return Some(deserialized_cache);
                } else {
                    eprintln!("unable to deserialize cache");
                }
            }
        } else {
            eprintln!("unable to make redis query");
        }
    } else {
        eprintln!("unable to get redis connection from pool");
    }

    let mut all_colleges: Vec<CollegeStruct> = Vec::new();
    let awc_client = Client::default();

    // The first request will give us the total number of colleges.
    let first_req_query = [("limit", 100), ("offset", 0)];
    let (total_count, mut first_list)= match awc_client.get("https://public.opendatasoft.com/api/explore/v2.1/catalog/datasets/us-colleges-and-universities/records").query(&first_req_query).unwrap().send().await {
        Ok(mut response) => {
            let response_body = match response.body().await {
                Ok(body) => body,
                Err(e) => {
                    eprintln!("error: {e}");
                    return None;
                }
            };
            match serde_json::from_slice::<CollegeAPIResp>(&response_body) {
                Ok(deserialized_resp) => (deserialized_resp.total_count, deserialized_resp.results),
                Err(e) => {
                    eprintln!("error: {e}");
                    return None;
                }
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            return None;
        }
    };

    // Now, we must append the result to the main vector.
    all_colleges.reserve(total_count);
    all_colleges.append(&mut first_list);

    let mut offset: i32 = 100;

    // Now, we need to keep going until we have all of them.
    while all_colleges.len() < total_count {
        let req_query = [("limit", 100), ("offset", offset)];
        let mut resp = match awc_client.get("https://public.opendatasoft.com/api/explore/v2.1/catalog/datasets/us-colleges-and-universities/records").query(&req_query).unwrap().send().await {
            Ok(mut response) => {
                let response_body = match response.body().await {
                    Ok(body) => body,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return None;
                    }
                };
                match serde_json::from_slice::<CollegeAPIResp>(&response_body) {
                    Ok(deserialized_resp) => deserialized_resp,
                    Err(e) => {
                        eprintln!("error: {e}");
                        return None;
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                return None;
            }
        };

        all_colleges.append(&mut resp.results);
        offset += 100;
    }

    // Once the colleges have been obtained, we can cache it for future use.
    if let Ok(serialized_colleges) = serde_json::to_string(&all_colleges) {
        if let Ok(mut redis_conn) = redis_pool.get().await {
            let _cache_store_resp = cmd("SET")
                .arg("@COLLEGE_LIST/CACHE")
                .arg(serialized_colleges)
                .arg("EX")
                .arg(COLLEGE_LIST_EXP)
                .query_async::<_, String>(&mut *redis_conn)
                .await;
        } else {
            eprintln!("unable to get redis connection while serializing");
        }
    } else {
        eprintln!("unable to serialize college data");
    }

    Some(all_colleges)
}
