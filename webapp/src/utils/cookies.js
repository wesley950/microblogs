import axios from "axios";
import Cookies from "js-cookie";

export function storeAuthToken(token) {
  Cookies.set("accessToken", token, {
    expires: 1,
    path: "/",
    sameSite: "strict",
  });
  axios.defaults.headers.common["Authorization"] = `Bearer ${token}`;
}
