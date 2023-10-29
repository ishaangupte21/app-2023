import { Formik } from "formik";
import { useState } from "react";
import {
  Button,
  Card,
  NumberInput,
  NumberInputData,
  Text,
  TextField,
  View,
} from "react-native-ui-lib";
import http from "../http";
import MapViewModal from "../components/MapViewModal";
import { ScrollView } from "react-native-gesture-handler";

export default function SearchColleges() {
  const [showSearchResults, setShowSearchResults] = useState<boolean>(false);
  const [searchResults, setSearchResults] = useState<College[]>([]);
  const [collegeOnMap, setCollegeOnMap] = useState<College | null>(null);
  const [showMapModal, setShowMapModal] = useState<boolean>(false);

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
        <Button label="Go Back" onPress={() => setShowSearchResults(false)} />
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
          <Button
            label="View on Map"
            style={{ marginTop: 8 }}
            onPress={() => {
                setCollegeOnMap(college);
                setShowMapModal(true);
            }}
          />
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
      <Text style={{ margin: "auto" }} text50M>
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
            />

            <View style={{ display: "flex" }}>
              <Text>Within </Text>
              <NumberInput
                initialNumber={50}
                fractionDigits={0}
                onChangeNumber={(data: NumberInputData) =>
                  (values.maxDistance = data)
                }
              />
              <Text> miles from </Text>
              <TextField
                placeholder="Starting Point"
                floatingPlaceholder
                onChangeText={handleChange("startingPoint")}
                onBlur={handleBlur("startingPoint")}
                value={values.startingPoint}
              />
            </View>

            <Button
              label="Search"
              style={{ width: "75%", marginTop: 10 }}
              onPress={() => handleSubmit()}
            />
          </View>
        )}
      </Formik>
    </View>
  );
}
