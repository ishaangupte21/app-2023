import { useRoute } from "@react-navigation/native";
import { StackScreenProps } from "@react-navigation/stack";
import {
  Card,
  Colors,
  ListItem,
  LoaderScreen,
  Text,
  View,
} from "react-native-ui-lib";
import { RootStackParamList } from "../types/RootStackParamList";
import http from "../http";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { ScrollView } from "react-native-gesture-handler";

type Props = StackScreenProps<RootStackParamList, "CollegeInfoView">;

interface CollegeData {
  admissions_url: string;
  apply_url: string;
  finaid_url: string;
  admission_info: {
    total_applicants: string;
    total_male_applicants: string;
    total_female_applicants: string;
    total_percent_admitted: string;
    total_percent_males_admitted: string;
    total_percent_females_admitted: string;
    sat_avg_english: string;
    sat_avg_math: string;
    act_avg: string;
  };
  application_reqs: string[];
}

export default function CollegeInfoView({ route, navigation }: Props) {
  const [collegeData, setCollegeData] = useState<CollegeData | null>(null);
  const [howReviewed, setHowReviewed] = useState<string | null>(null);

  const getCollegeData = async () => {
    const { data } = await http.get(
      `/college/info/${route.params.college.ipedsid}?name=${route.params.college.name}`
    );

    return data.college;
  };

  const getHowReviewed = async () => {
    const { data } = await http.get(
      `/colleges/how-reviewed?name=${route.params.college.name}`
    );
    return data.how_reviewed;
  };

  useEffect(() => {
    getCollegeData()
      .then((data) => setCollegeData(data))
      .catch((err) => console.error(err));

    getHowReviewed()
      .then((data) => setHowReviewed(data))
      .catch((err) => console.error(err));
  }, []);

  return collegeData != null && howReviewed != null ? (
    <ScrollView
      style={{
        width: "100%",
        height: "100%",
        paddingTop: 20,
        paddingHorizontal: 20,
      }}
    >
      <Text text40 style={{ margin: "auto", marginBottom: 10 }}>
        {route.params.college.name}
      </Text>

      <Card style={{ padding: 20 }}>
        <Text text50 style={{ marginBottom: 10 }}>
          Admission Information
        </Text>
        <Text text70M color={Colors.green30}>
          Admissions Page:{" "}
        </Text>
        <Text text70>{collegeData.admissions_url}</Text>
        <Text text70M color={Colors.green30}>
          Applications Page:{" "}
        </Text>
        <Text text70>{collegeData?.apply_url}</Text>
        <Text text70M color={Colors.green30}>
          Financial Aid Page:{" "}
        </Text>
        <Text text70>{collegeData?.finaid_url}</Text>
      </Card>

      <Card style={{ padding: 20, marginTop: 30 }}>
        <Text text50 style={{ marginBottom: 10 }}>
          Admission Statistics
        </Text>
        <Text text70M color={Colors.blue20}>
          Total Applicants
        </Text>
        <Text text70>{collegeData.admission_info.total_applicants}</Text>
        <Text text70M color={Colors.blue20}>
          Total Male Applicants
        </Text>
        <Text text70>{collegeData.admission_info.total_male_applicants}</Text>
        <Text text70M color={Colors.blue20}>
          Total Female Applicants
        </Text>
        <Text text70>{collegeData.admission_info.total_female_applicants}</Text>
        <Text text70M color={Colors.blue20}>
          Total Percent Admitted
        </Text>
        <Text text70>{collegeData.admission_info.total_percent_admitted}</Text>
        <Text text70M color={Colors.blue20}>
          Total Percent of Males Admitted
        </Text>
        <Text text70>
          {collegeData.admission_info.total_percent_males_admitted}
        </Text>
        <Text text70M color={Colors.blue20}>
          Total Percent of Females Admitted
        </Text>
        <Text text70>
          {collegeData.admission_info.total_percent_females_admitted}
        </Text>
        <Text text70M color={Colors.blue20}>
          Median SAT English Score
        </Text>
        <Text text70>{collegeData.admission_info.sat_avg_english}</Text>
        <Text text70M color={Colors.blue20}>
          Median SAT Math Score
        </Text>
        <Text text70>{collegeData.admission_info.sat_avg_math}</Text>
        <Text text70M color={Colors.blue20}>
          Median ACT Score
        </Text>
        <Text text70>{collegeData.admission_info.act_avg}</Text>
      </Card>

      <Card style={{ padding: 20, marginTop: 30 }}>
        <Text text50 style={{ marginBottom: 10 }}>
          Application Requirements
        </Text>
        {collegeData.application_reqs.map((req, index) => (
          <Text
            text70M
            color={Colors.red40}
            style={{ marginBottom: 6 }}
            key={index}
          >
            - {req}
          </Text>
        ))}
      </Card>

      <Card style={{ padding: 20, marginTop: 30 }}>
        <Text text50 style={{ marginBottom: 10 }}>
          How does this college review applications?
        </Text>
        <Text text70M color={Colors.purple30}>
          {howReviewed}
        </Text>
      </Card>
    </ScrollView>
  ) : (
    <View>
      <LoaderScreen
        style={{ marginTop: 700 }}
        message="Loading..."
        color={Colors.green40}
      />
    </View>
  );
}
