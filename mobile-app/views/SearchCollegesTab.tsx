import { Formik } from "formik";
import { useState } from "react";
import {
  Button,
  Card,
  Colors,
  NumberInput,
  NumberInputData,
  Text,
  TextField,
  View,
} from "react-native-ui-lib";
import http from "../http";
import MapViewModal from "../components/MapViewModal";
import { ScrollView } from "react-native-gesture-handler";
import { FontAwesomeIcon } from "@fortawesome/react-native-fontawesome";
import { faMap, faPlus } from "@fortawesome/free-solid-svg-icons";
import { useNavigation } from "@react-navigation/native";
import { StackNavigationProp } from "@react-navigation/stack";

export default function SearchColleges() {
  const [showSearchResults, setShowSearchResults] = useState<boolean>(false);
  const [searchResults, setSearchResults] = useState<College[]>([]);
  const [collegeOnMap, setCollegeOnMap] = useState<College | null>(null);
  const [showMapModal, setShowMapModal] = useState<boolean>(false);

  const navigation = useNavigation<StackNavigationProp<any>>();

  const handleFormSubmit = async (values: any) => {
    const { data } = await http.get("/colleges/with-params", {
      params: {
        name: values.name,
        max_distance: values.maxDistance.userInput,
        starting_point: values.startingPoint,
      },
    });

    setSearchResults(data.colleges);
    setShowSearchResults(true);
  };

  return showSearchResults ? (
    <ScrollView
      style={{
        paddingVertical: 20,
        paddingHorizontal: 10,
      }}
    >
      <View>
        <Button borderRadius={8} backgroundColor={Colors.red40} label="Go Back" onPress={() => setShowSearchResults(false)} />
      </View>
      {searchResults.map((college) => (
        <Card
          key={college.ipedsid}
          style={{ marginVertical: 10, marginHorizontal: 15, padding: 20 }}
        >
          <Text text60M style={{ marginBottom: 6 }}>
            {college.name}
          </Text>
          <Text text80>
            {college.address} {college.city}, {college.state} {college.zip}
          </Text>
          <View style={{ display: "flex", flexDirection: "row", justifyContent: "space-evenly", marginTop: 20}}>
            <Button
              iconSource={() => (
                <FontAwesomeIcon icon={faMap} color="white" size={28} />
              )}
              style={{
                marginTop: 8,
                paddingHorizontal: 20,
                paddingVertical: 10,
              }}
              backgroundColor={Colors.blue40}
              borderRadius={10}
              onPress={() => {
                setCollegeOnMap(college);
                setShowMapModal(true);
              }}
            />
            <Button
              iconSource={() => (
                <FontAwesomeIcon color="white" size={28} icon={faPlus} />
              )}
              backgroundColor={Colors.green40}
              borderRadius={10}
              style={{
                marginTop: 8,
                paddingHorizontal: 20,
                paddingVertical: 10,
              }}
            />

            <Button
              label="More"
              onPress={() =>
                navigation.push("CollegeInfoView", {
                  college,
                })
              }

              borderRadius={10}
              backgroundColor={Colors.orange30}
            />
          </View>
        </Card>
      ))}

      <MapViewModal
        college={collegeOnMap}
        visible={showMapModal}
        onBackgroundPress={() => setShowMapModal(false)}
      />
    </ScrollView>
  ) : (
    <View
      style={{
        width: "100%",
        height: "100%",
        paddingVertical: 20,
        paddingHorizontal: 10,
      }}
    >
      <Card style={{ padding: 20, marginTop: 25 }}>
        <Text style={{ marginLeft: 65, marginTop: 20 }} text40M>
          Search for Colleges
        </Text>
        <Formik
          initialValues={{
            name: "",
            maxDistance: { userInput: "50" },
            startingPoint: "",
          }}
          onSubmit={(values) => handleFormSubmit(values)}
        >
          {({
            handleChange,
            handleBlur,
            handleSubmit,
            values,
            setFieldValue,
          }) => (
            <View style={{ margin: "auto" }}>
              <TextField
                placeholder="Name"
                floatingPlaceholder
                onChangeText={handleChange("name")}
                onBlur={handleBlur("name")}
                value={values.name}
                text70L
              />

              <View style={{ display: "flex" }}>
                <Text text70M style={{ marginTop: 8 }}>
                  Within{" "}
                </Text>
                <NumberInput
                  initialNumber={50}
                  fractionDigits={0}
                  onChangeNumber={(data: NumberInputData) =>
                    (values.maxDistance = data)
                  }
                />
                <Text text70M style={{ marginTop: 8 }}>
                  {" "}
                  Miles From{" "}
                </Text>
                <TextField
                  placeholder="Starting Point"
                  floatingPlaceholder
                  onChangeText={handleChange("startingPoint")}
                  onBlur={handleBlur("startingPoint")}
                  value={values.startingPoint}
                  text70L
                />
              </View>

              <Button
                label="Search"
                style={{ width: "75%", marginTop: 20, marginLeft: 50 }}
                onPress={() => handleSubmit()}
                borderRadius={8}
                text70M
                backgroundColor={Colors.blue30}
              />
            </View>
          )}
        </Formik>
      </Card>
    </View>
  );
}
