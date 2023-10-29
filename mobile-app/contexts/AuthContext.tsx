import { createContext, useContext, useEffect, useState } from "react";
import http from "../http";
import AsyncStorage from "@react-native-async-storage/async-storage";

interface AuthProps {
  accessToken: string | null;
  isAuthenticated: boolean;
  login?: (googleToken: string) => Promise<void>;
  logout?: () => Promise<void>;
}

const AuthContext = createContext<AuthProps>({
  accessToken: null,
  isAuthenticated: false,
});

export const useAuth = () => useContext(AuthContext);

export default function AuthProvider({ children }: any) {
  const [isAuthenticated, setAuthenticated] = useState<boolean>(false);
  const [accessToken, setAccessToken] = useState<string | null>(null);

  const storeAccessToken = async (accessToken: string) => {
    await AsyncStorage.setItem("@auth/access-token", accessToken);
    setAccessToken(accessToken);
  };

  const removeAccessToken = async () => {
    await AsyncStorage.removeItem("@auth/access-token");
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
      `/verify-access-token?token=${accessToken}`
    );
    if (!data.valid_token) {
      await removeAccessToken();
    } else {
      setAccessToken(accessToken);
      setAuthenticated(true);
    }
  };

  useEffect(() => {
    verifyAccessToken().catch((err) => {
      removeAccessToken();
    });
  }, []);

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
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}
