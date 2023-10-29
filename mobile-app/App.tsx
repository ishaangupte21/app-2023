import "react-native-gesture-handler";
import { NavigationContainer } from "@react-navigation/native";
import { createStackNavigator } from "@react-navigation/stack";
import AuthView from "./views/AuthView";
import AuthProvider from "./contexts/AuthContext";
import DashView from "./views/DashView";
import DashHeader from "./components/DashHeader";

const Stack = createStackNavigator();

export default function App() {
  return (
    <AuthProvider>
      <NavigationContainer>
        <Stack.Navigator initialRouteName="AuthView">
          <Stack.Screen name="AuthView" component={AuthView} />
          <Stack.Screen
            name="DashView"
            component={DashView}
            options={{header: () => null}}
          />
        </Stack.Navigator>
      </NavigationContainer>
    </AuthProvider>
  );
}
