import axios from "axios";
import Cookies from "js-cookie";

export const authProvider = {
  isAuthenticated: false,
};

export function storeAuthToken(token) {
  Cookies.set("accessToken", token, {
    expires: 1,
    path: "/",
    sameSite: "strict",
  });
  authProvider.isAuthenticated = true;
  axios.defaults.headers.common["Authorization"] = `Bearer ${token}`;
}
