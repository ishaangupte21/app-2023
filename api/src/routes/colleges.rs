// Routes under the /colleges path

use std::{f64::consts::PI, time::Duration, str::FromStr};

use actix_web::{get, web, HttpResponse};
use awc::Client;
use bb8_redis::{bb8, redis::cmd, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use tl::ParserOptions;

use crate::{
    app_state::AppState,
    structures::{CollegeCoord, CollegeStruct},
};

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
    let (total_count, mut first_list)= match awc_client.get("https://public.opendatasoft.com/api/explore/v2.1/catalog/datasets/us-colleges-and-universities/records?where=naics_desc%20like%20%22COLLEGES%2C%20UNIVERSITIES%2C%20AND%20PROFESSIONAL%20SCHOOLS%22&limit=100&offset=0").send().await {
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
        let mut resp = match awc_client.get(format!("https://public.opendatasoft.com/api/explore/v2.1/catalog/datasets/us-colleges-and-universities/records?where=naics_desc%20like%20%22COLLEGES%2C%20UNIVERSITIES%2C%20AND%20PROFESSIONAL%20SCHOOLS%22&limit=100&offset={offset}")).send().await {
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

#[derive(Deserialize)]
pub struct CollegeParamReqQuery {
    pub name: Option<String>,
    pub max_distance: Option<String>,
    pub starting_point: Option<String>,
}

#[get("/colleges/with-params")]
pub async fn handle_get_colleges_with_params(
    state: web::Data<AppState>,
    query: web::Query<CollegeParamReqQuery>,
) -> HttpResponse {
    // Now, we must get all the colleges.
    let mut college_list = match get_all_colleges(&state.redis_pool).await {
        Some(college_list) => college_list,
        None => return HttpResponse::InternalServerError().json(CollegeListResp::empty()),
    };

    // First, if a name is present, we must filter through the college list.
    if let Some(name_fragment) = &query.name {
        let college_list_iter = college_list.into_iter();
        let filtered = college_list_iter.filter(|college| {
            college
                .name
                .to_lowercase()
                .contains(&name_fragment.to_lowercase())
        });
        college_list = filtered.collect();
    }

    // Now, we need to check for a within
    if let Some(max_distance) = &query.max_distance {
        if let Some(starting_point) = &query.starting_point {
            let college_list_iter = college_list.into_iter();
            let max_distance = match max_distance.parse::<f64>() {
                Ok(val) => val,
                Err(_) => return HttpResponse::BadRequest().json(CollegeListResp::empty()),
            };

            let starting_point_coords =
                match get_geolocation_coords(starting_point, &state.pos_stack_key).await {
                    Some(coords) => coords,
                    None => {
                        return HttpResponse::InternalServerError().json(CollegeListResp::empty())
                    }
                };

            let filtered = college_list_iter.filter(|college| {
                calculate_distance_between_coords(&college.geo_point_2d, &starting_point_coords)
                    < max_distance
            });
            college_list = filtered.collect();
        }
    }

    HttpResponse::Ok().json(CollegeListResp::from(college_list))
}

const R_EARTH: f64 = 3956.0;

fn calculate_distance_between_coords(p1: &CollegeCoord, p2: &CollegeCoord) -> f64 {
    let lat1 = convert_to_radians(p1.lat);
    let long1 = convert_to_radians(p1.lon);
    let lat2 = convert_to_radians(p2.lat);
    let long2 = convert_to_radians(p2.lon);

    // Haversine formula
    let dlong = long2 - long1;
    let dlat = lat2 - lat1;

    let half_dlat = dlat / 2.0;
    let half_dlong = dlong / 2.0;

    let result = half_dlat.sin().powi(2) + lat1.cos() * lat2.cos() * half_dlong.sin().powi(2);
    let result = 2.0 * result.sqrt().asin();

    result * R_EARTH
}

const ONE_DEG_TO_RAD: f64 = PI / 180.0;

#[inline(always)]
fn convert_to_radians(angle: f64) -> f64 {
    angle * ONE_DEG_TO_RAD
}

#[derive(Deserialize)]
pub struct PosStackResp {
    pub data: Vec<PosStackCoord>,
}

#[derive(Deserialize)]
pub struct PosStackCoord {
    pub latitude: f64,
    pub longitude: f64,
}

async fn get_geolocation_coords(location: &str, position_stack_key: &str) -> Option<CollegeCoord> {
    let awc_client = Client::default();

    let query = [
        ("access_key", position_stack_key),
        ("query", location),
        ("output", "json"),
        ("limit", "1"),
    ];

    match awc_client
        .get("http://api.positionstack.com/v1/forward")
        .query(&query)
        .unwrap()
        .send()
        .await
    {
        Ok(mut resp) => {
            let resp_body = match resp.body().await {
                Ok(body) => body,
                Err(e) => {
                    eprintln!("error: {e}");
                    return None;
                }
            };

            match serde_json::from_slice::<PosStackResp>(&resp_body) {
                Ok(val) => {
                    let coord_obj = &val.data[0];
                    Some(CollegeCoord {
                        lon: coord_obj.longitude,
                        lat: coord_obj.latitude,
                    })
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            None
        }
    }
}

#[derive(Serialize)]
pub struct GetSingleCollegeRespWrapper<'a> {
    college: Option<GetSingleCollegeResp>,
    msg: Option<&'a str>,
}

impl GetSingleCollegeRespWrapper<'_> {
    pub fn from_msg<'a>(msg: &'a str) -> GetSingleCollegeRespWrapper<'a> {
        GetSingleCollegeRespWrapper {
            msg: Some(msg),
            college: None,
        }
    }

    pub fn from_college_data<'a>(college: GetSingleCollegeResp) -> GetSingleCollegeRespWrapper<'a> {
        GetSingleCollegeRespWrapper {
            college: Some(college),
            msg: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GetSingleCollegeResp {
    admissions_url: String,
    apply_url: String,
    finaid_url: String,
    admission_info: CollegeAdmissionInfo,
    application_reqs: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct CollegeAdmissionInfo {
    total_applicants: String,
    total_male_applicants: String,
    total_female_applicants: String,
    total_percent_admitted: String,
    total_percent_males_admitted: String,
    total_percent_females_admitted: String,
    sat_avg_english: String,
    sat_avg_math: String,
    act_avg: String,
}

#[derive(Deserialize)]
pub struct GetSingleCollegeQuery {
    pub name: String,
}

#[get("/college/info/{ipedsid}")]
pub async fn handle_get_single_college_info(
    path: web::Path<String>,
    query: web::Query<GetSingleCollegeQuery>,
    data: web::Data<AppState>,
) -> HttpResponse {
    // If the cache is missed, we must first get the data.
    let mut redis_conn = data.redis_pool.get().await.unwrap();
    let redis_resp: Option<String> = cmd("GET")
        .arg(format!("COLLEGE_DATA_{}", path))
        .query_async(&mut *redis_conn)
        .await
        .unwrap();

    if let Some(resp) = redis_resp {
        let parsed = serde_json::from_str::<GetSingleCollegeResp>(&resp).unwrap();
        return HttpResponse::Ok().json(GetSingleCollegeRespWrapper::from_college_data(parsed));
    }

    let awc_client = Client::default();
    let req_query = [("id", path.as_str())];
    let resp_contents = match awc_client
        .get("https://nces.ed.gov/collegenavigator")
        .query(&req_query)
        .unwrap()
        .send()
        .await
    {
        Ok(mut resp) => match resp.body().await {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("error: {e}");
                return HttpResponse::InternalServerError().json(
                    GetSingleCollegeRespWrapper::from_msg("Unable to parse html response"),
                );
            }
        },
        Err(e) => {
            eprintln!("error: {e}");
            return HttpResponse::InternalServerError().json(
                GetSingleCollegeRespWrapper::from_msg("Unable to fetch html data"),
            );
        }
    };

    let dom = match tl::parse(
        unsafe { std::str::from_utf8_unchecked(&resp_contents) },
        ParserOptions::default(),
    ) {
        Ok(dom) => dom,
        Err(e) => {
            eprintln!("error: {e}");
            return HttpResponse::InternalServerError().json(
                GetSingleCollegeRespWrapper::from_msg("Unable to parse html data"),
            );
        }
    };

    let dom_parser = dom.parser();

    let mut general_info_handle = match dom
        .query_selector("div#divctl00_cphCollegeNavBody_ucInstitutionMain_dtpGeneral")
    {
        Some(el) => el,
        None => {
            return HttpResponse::InternalServerError().json(GetSingleCollegeRespWrapper::from_msg(
                "Unable to get general info handle from html data",
            ))
        }
    };

    let children = general_info_handle
        .next()
        .unwrap()
        .get(dom_parser)
        .unwrap()
        .children()
        .unwrap()
        .all(dom_parser);
    //let (admissions_url, apply_url, finaid_url) = get_general_info_urls(general_info_handle, dom_parser);
    let admissions_url = children[9].inner_html(dom_parser).to_string();
    let apply_url = children[16].inner_html(dom_parser).to_string();
    let finaid_url = children[24].inner_html(dom_parser).to_string();

    let finaid_el_handle = match dom.get_element_by_id("finaid") {
        Some(el) => el,
        None => {
            return HttpResponse::InternalServerError().json(GetSingleCollegeRespWrapper::from_msg(
                "Unable to get finaid handle from html data",
            ))
        }
    };

    let finaid_el = match finaid_el_handle.get(dom_parser) {
        Some(el) => el,
        None => {
            return HttpResponse::InternalServerError().json(GetSingleCollegeRespWrapper::from_msg(
                "Unable to get finaid from html data",
            ))
        }
    };

    let admissions_el_handle = match dom
        .get_element_by_id("divctl00_cphCollegeNavBody_ucInstitutionMain_ctl04")
    {
        Some(el) => el,
        None => {
            return HttpResponse::InternalServerError().json(GetSingleCollegeRespWrapper::from_msg(
                "Unable to get applications handle from html data",
            ))
        }
    };

    let applications_el = match admissions_el_handle.get(dom_parser) {
        Some(el) => el,
        None => {
            return HttpResponse::InternalServerError().json(GetSingleCollegeRespWrapper::from_msg(
                "Unable to get applications from html data",
            ))
        }
    };

    let admissions_html = applications_el
        .inner_html(dom_parser)
        .to_string()
        .replace("\"", "\\\"");
    let admissions_body_req = serde_json::json!({
        "input": admissions_html,
    });
    let mut get_admissions_data_request = awc_client
        .post("http://localhost:8001/get-application-statistics")
        .timeout(Duration::from_secs(30))
        .send_json(&admissions_body_req)
        .await
        .unwrap();

    let get_admissions_response_body = get_admissions_data_request.body().await.unwrap();

    let college_admission_info =
        serde_json::from_slice::<CollegeAdmissionInfo>(&get_admissions_response_body).unwrap();

    let reqs_body_req = serde_json::json!({
        "name": query.name
    });

    let mut get_req_data_req = awc_client
        .post("http://localhost:8001/get-application-requirements")
        .timeout(Duration::from_secs(30))
        .send_json(&reqs_body_req)
        .await
        .unwrap();

    let get_req_data_body = get_req_data_req.body().await.unwrap();

    let college_reqs = serde_json::from_slice::<Vec<String>>(&get_req_data_body).unwrap();

    let resp = GetSingleCollegeResp {
        admissions_url,
        apply_url,
        finaid_url,
        admission_info: college_admission_info,
        application_reqs: college_reqs,
    };

    // Cache it.
    let _result: Option<String> = cmd("SET")
        .arg(format!("COLLEGE_DATA_{}", path))
        .arg(serde_json::to_string(&resp).unwrap())
        .arg("EX")
        .arg(COLLEGE_LIST_EXP)
        .query_async(&mut *redis_conn)
        .await
        .unwrap();
    HttpResponse::Ok().json(GetSingleCollegeRespWrapper::from_college_data(resp))
}

#[derive(Deserialize, Serialize)]
pub struct HowReviewedResp {
    how_reviewed: String,
}

#[get("/colleges/how-reviewed")]
pub async fn handle_how_reviewed_route(
    query: web::Query<GetSingleCollegeQuery>,
    data: web::Data<AppState>,
) -> HttpResponse {
    let mut redis_conn = data.redis_pool.get().await.unwrap();
    let redis_resp: Option<String> = cmd("GET")
        .arg(format!("COLLEGE_REVIEWED_NAME_{}", query.name))
        .query_async(&mut *redis_conn)
        .await
        .unwrap();

    if let Some(resp) = redis_resp {
        let parsed = serde_json::from_str::<HowReviewedResp>(&resp).unwrap();
        return HttpResponse::Ok().json(parsed);
    }

    let mut how_reviewed_req = serde_json::json!({
        "name": query.name
    });

    let how_reviewed_req = Client::default()
        .post("http://localhost:8001/get-how-reviewed")
        .timeout(Duration::from_secs(30))
        .send_json(&how_reviewed_req)
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

        let parsed_resp = HowReviewedResp {how_reviewed: String::from_str(unsafe {std::str::from_utf8_unchecked(&how_reviewed_req)}).unwrap()};

    let _result: Option<String> = cmd("SET")
        .arg(format!("COLLEGE_REVIEWED_NAME_{}", query.name))
        .arg(serde_json::to_string(&parsed_resp).unwrap())
        .arg("EX")
        .arg(COLLEGE_LIST_EXP)
        .query_async(&mut *redis_conn)
        .await
        .unwrap();

    HttpResponse::Ok().json(parsed_resp)
}
