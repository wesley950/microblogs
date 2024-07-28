import { createBrowserRouter, RouterProvider } from "react-router-dom";

import Root, { loader as rootLoader } from "./routes/root";
import ErrorPage from "./error-page";
import Login, { action as loginAction } from "./routes/login";
import Register, { action as registerAction } from "./routes/register";
import Index, { action as indexAction } from "./routes";
import Post, { action as postAction } from "./routes/post";
import { likeAction } from "./components/interactions";

const router = createBrowserRouter([
  {
    id: "root",
    path: "/",
    element: <Root />,
    loader: rootLoader,
    errorElement: <ErrorPage />,
    children: [
      {
        index: true,
        element: <Index />,
        action: indexAction,
      },
      {
        path: "post/:postUuid",
        element: <Post />,
        action: postAction,
      },
      {
        path: "post/:postUuid/like",
        action: likeAction,
      },
      {
        path: "entrar",
        element: <Login />,
        action: loginAction,
      },
      {
        path: "registrar",
        element: <Register />,
        action: registerAction,
      },
    ],
  },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
