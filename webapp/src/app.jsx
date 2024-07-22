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
import Index, { loader as indexLoader } from "./routes";
import Post, {
  loader as postLoader,
  action as postAction,
} from "./components/post";

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
        loader: indexLoader,
      },
      {
        path: "post/:postId",
        element: <Post />,
        loader: postLoader,
        action: postAction,
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
