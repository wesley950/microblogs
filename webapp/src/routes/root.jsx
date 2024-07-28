import axios from "axios";
import { Navigate, Outlet, useLoaderData, useLocation } from "react-router-dom";

import Cookies from "js-cookie";
import Navbar from "../components/navbar";
import { storeAuthToken } from "../utils/auth";

export async function loader() {
  axios.defaults.baseURL = import.meta.env.VITE_API_BASE_ADDRESS;
  axios.defaults.headers.common["Content-Type"] = "application/json";

  let accessToken = Cookies.get("accessToken");
  if (accessToken !== undefined) {
    axios.defaults.headers.common["Authorization"] = `Bearer ${accessToken}`;
    try {
      let res = await axios.get("/users/refresh_access");
      if (res.status === 200) {
        storeAuthToken(res.data.token);
        return {
          isAuthenticated: true,
        };
      }
    } catch (error) {
      console.error(error);
    }
  }

  return {
    isAuthenticated: false,
  };
}

export default function Root() {
  const { isAuthenticated } = useLoaderData();
  const location = useLocation();

  if (
    !isAuthenticated &&
    location.pathname !== "/entrar" &&
    location.pathname !== "/registrar"
  ) {
    return <Navigate to="entrar" replace />;
  }

  return (
    <>
      <Navbar />
      <Outlet />
    </>
  );
}
