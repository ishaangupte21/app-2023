import { createContext, useContext, useEffect, useState } from "react";
import http from "../http";
import AsyncStorage from "@react-native-async-storage/async-storage";

interface AuthProps {
  accessToken: string | null;
  isAuthenticated: boolean;
  userData: User | null;
  login?: (googleToken: string) => Promise<void>;
  logout?: () => Promise<void>;
}

const AuthContext = createContext<AuthProps>({
  accessToken: null,
  isAuthenticated: false,
  userData: null,
});

export const useAuth = () => useContext(AuthContext);

interface User {
  id: number;
  email: string;
  name: string;
  picture: string;
}

export default function AuthProvider({ children }: any) {
  const [isAuthenticated, setAuthenticated] = useState<boolean>(false);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [userData, setUserData] = useState<User | null>(null);

  const storeAccessToken = async (accessToken: string) => {
    await AsyncStorage.setItem("@auth/access-token", accessToken);
    http.defaults.headers.common["Authorization"] = `Bearer ${accessToken}`;
    setAccessToken(accessToken);
  };

  const removeAccessToken = async () => {
    await AsyncStorage.removeItem("@auth/access-token");
    http.defaults.headers.common["Authorization"] = "";
    setAccessToken(null);
    setAuthenticated(false);
  };

  const verifyAccessToken = async () => {
    const accessToken = await AsyncStorage.getItem("@auth/access-token");
    if (!accessToken) {
      setAuthenticated(false);
      return;
    }

    const { data } = await http.get(
      `/auth/verify-access-token?token=${accessToken}`
    );

    if (!data.claims) {
      await removeAccessToken();
    } else {
      setAccessToken(accessToken);
      setUserData(data.claims);
      setAuthenticated(true);
    }
  };

  useEffect(() => {
    verifyAccessToken().catch((err) => {
      removeAccessToken();
    });
  }, []);

  useEffect(() => {
    console.log(isAuthenticated);
    if (isAuthenticated) {
      verifyAccessToken().catch((err) => {
        console.error(err);
        removeAccessToken();
      });
    }
  }, [isAuthenticated]);

  const login = async (googleToken: string) => {
    const { data } = await http.post("/auth/google-login", {
      gid_token: googleToken,
    });

    await storeAccessToken(data.access_token);
    setAuthenticated(true);
  };

  const logout = async () => {};

  return (
    <AuthContext.Provider
      value={{
        accessToken,
        isAuthenticated,
        userData,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
