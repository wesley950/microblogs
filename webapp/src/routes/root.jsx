import axios from "axios";
import {
  Outlet,
  useLoaderData,
  useLocation,
  useNavigate,
} from "react-router-dom";

import Cookies from "js-cookie";
import { useEffect } from "react";
import Navbar from "../components/navbar";

export async function loader() {
  axios.defaults.baseURL = import.meta.env.VITE_API_BASE_ADDRESS;
  axios.defaults.headers.common["Content-Type"] = "application/json";

  let accessToken = Cookies.get("accessToken");
  if (accessToken !== undefined) {
    axios.defaults.headers.common["Authorization"] = `Bearer ${accessToken}`;
    try {
      let res = await axios.get("/users/refresh_access");
      if (res.status === 200) {
        Cookies.set("accessToken", res.data.token);
        axios.defaults.headers.common[
          "Authorization"
        ] = `Bearer ${res.data.token}`;
        return {
          isLogged: true,
        };
      }
    } catch (error) {
      console.error(error);
    }
  }

  return {
    isLogged: false,
  };
}

export default function Root() {
  const { isLogged } = useLoaderData();
  const location = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (
      !isLogged &&
      location.pathname !== "/entrar" &&
      location.pathname !== "/registrar"
    ) {
      navigate("/entrar");
    }
  }, [isLogged]);

  return (
    <>
      <Navbar />
      <Outlet />
    </>
  );
}
