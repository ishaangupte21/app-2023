import { View } from "react-native-ui-lib";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createBottomTabNavigator } from "@react-navigation/bottom-tabs";
import MyCollegesTab from "./MyCollegesTab";
import DashHeader from "../components/DashHeader";
import SearchColleges from "./SearchCollegesTab";

// Initialize a react query client to be used from here.
const queryClient = new QueryClient();

// Create the bottom tab navigator.
const Tab = createBottomTabNavigator();

export default function DashView() {
  return (
    <View style={{ width: "100%", height: "100%" }}>
      <QueryClientProvider client={queryClient}>
        <Tab.Navigator initialRouteName="MyColleges">
          <Tab.Screen name="MyColleges" component={MyCollegesTab} />
          <Tab.Screen name="SearchColleges" component={SearchColleges} />
        </Tab.Navigator>
      </QueryClientProvider>
    </View>
  );
}
