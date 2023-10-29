import { View, Text, Button } from "react-native-ui-lib";
import { GoogleSignin } from "@react-native-google-signin/google-signin";
import { useAuth } from "../contexts/AuthContext";
import { useEffect } from "react";
import { useNavigation } from "@react-navigation/native";
import { StackNavigationProp } from "@react-navigation/stack";

GoogleSignin.configure({
  scopes: ["openid", "profile", "email"],
  iosClientId: process.env.EXPO_PUBLIC_GOOGLE_IOS_CLIENT_ID,
});

export default function AuthView() {
  const { login, isAuthenticated} = useAuth();

  const navigation = useNavigation<StackNavigationProp<any>>();

  const handleLoginButtonPress = async () => {
    try {
      const { idToken } = await GoogleSignin.signIn();
      if (!idToken) throw new Error();
      await login!(idToken);
    } catch (err) {
      console.error(err);
    }
  };

  useEffect(() => {
    if(isAuthenticated) {
      navigation.push('DashView');
    }
  }, [isAuthenticated]);

  return (
    <View>
      <Text text50>Google Login Screen</Text>
      <Button
        label={"Login with Google"}
        onPress={() => handleLoginButtonPress()}
      />
    </View>
  );
}
