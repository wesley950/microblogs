import axios from "axios";
import { Form, Link, redirect } from "react-router-dom";

import Cookies from "js-cookie";

export async function loader() {
  let accessToken = Cookies.get("accessToken");
  if (accessToken !== undefined) {
    return redirect("/");
  }

  return null;
}

export async function action({ request }) {
  const formData = await request.formData();
  const data = JSON.stringify({
    username: formData.get("username"),
    password: formData.get("password"),
  });

  try {
    let res = await axios.post("/users/login", data);
    if (res.status === 200) {
      Cookies.set("accessToken", res.data.token);
      axios.defaults.headers.common[
        "Authorization"
      ] = `Bearer ${res.data.token}`;
      return redirect("/");
    }
  } catch (error) {
    console.log(error);
  }

  // TODO: place error message in query string
  return redirect("/entrar");
}

export default function Login() {
  return (
    <div className="container vh-100 d-flex justify-content-center align-items-center">
      <Form method="post" className="vstack gap-2 my-auto">
        <h3>Entrar no Microblogs</h3>
        <input
          type="text"
          name="username"
          id="username"
          placeholder="Nome da conta"
          className="form-control"
        />
        <input
          type="password"
          name="password"
          id="password"
          placeholder="Senha"
          className="form-control"
        />
        <button type="submit" className="btn btn-primary">
          Entrar
        </button>
        <p className="text-muted text-center">
          Ainda não tem uma conta? Registre-se <Link to="/registrar">aqui</Link>
          .
        </p>
      </Form>
    </div>
  );
}
