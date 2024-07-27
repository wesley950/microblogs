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
import Index, { action as indexAction } from "./routes";
import Post, { action as postAction } from "./routes/post";
import { likeAction } from "./components/interactions";

const router = createBrowserRouter([
  {
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
        path: "post/:postId",
        element: <Post />,
        action: postAction,
      },
      {
        path: "post/:postId/like",
        action: likeAction,
      },
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
