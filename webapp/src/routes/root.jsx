import axios from "axios";
import { Navigate, Outlet, useLoaderData, useLocation } from "react-router-dom";

import Cookies from "js-cookie";
import Navbar from "../components/navbar";
import { storeAuthToken } from "../utils/auth";

import Menu from "../components/menu";

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
          username: res.data.username,
          realName: res.data.real_name,
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
      <div className="row m-0">
        <div className="col">{isAuthenticated && <Menu />}</div>
        <div className="col border-start border-end">
          <Outlet />
        </div>
        <div className="col d-none d-sm-block"></div>
      </div>
    </>
  );
}
