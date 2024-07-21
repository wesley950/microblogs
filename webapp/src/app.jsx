import { createBrowserRouter, RouterProvider } from "react-router-dom";
import Root, { loader as rootLoader } from "./routes/root";
import ErrorPage from "./error-page";
import Login, {
  loader as loginLoader,
  action as loginAction,
} from "./routes/login";
import Register, {
  loader as registerLoader,
  action as registerAction,
} from "./routes/register";

const router = createBrowserRouter([
  {
    path: "/",
    element: <Root />,
    loader: rootLoader,
    errorElement: <ErrorPage />,
    children: [
      {
        path: "/entrar",
        element: <Login />,
        loader: loginLoader,
        action: loginAction,
      },
      {
        path: "/registrar",
        element: <Register />,
        loader: registerLoader,
        action: registerAction,
      },
    ],
  },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
