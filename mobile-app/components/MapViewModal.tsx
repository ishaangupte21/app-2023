import { FontAwesomeIcon } from "@fortawesome/react-native-fontawesome";
import MapView, { Marker } from "react-native-maps";
import { Button, Colors, Modal, Text } from "react-native-ui-lib";
import { faTimes } from "@fortawesome/free-solid-svg-icons";

interface Props {
  visible: boolean;
  college: College | null;
  onBackgroundPress: () => void;
}

export default function MapViewModal({
  visible,
  college,
  onBackgroundPress,
}: Props) {
  if (college != null) {
    return (
      <Modal visible={visible} onBackgroundPress={onBackgroundPress}>
        <Button
          round
          backgroundColor={Colors.red50}
          color={Colors.white}
          style={{
            position: "absolute",
            zIndex: 100,
            marginTop: "10%",
            marginLeft: "6%",
            height: 10,
            width: 25,
          }}
          iconSource={() => (
            <FontAwesomeIcon color="white" size={30} icon={faTimes} />
          )}
          onPress={onBackgroundPress}
        />
        <MapView
          style={{ width: "100%", height: "100%" }}
          initialRegion={{
            latitude: college.geo_point_2d.lat,
            longitude: college.geo_point_2d.lon,
            latitudeDelta: 0.0922,
            longitudeDelta: 0.0922,
          }}
        >
          <Marker
            coordinate={{
              latitude: college.geo_point_2d.lat,
              longitude: college.geo_point_2d.lon,
            }}
            title={college.name}
            description={college.naics_desc}
          />
        </MapView>
      </Modal>
    );
  } else return null;
}
