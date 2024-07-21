import { Form, Link, redirect } from "react-router-dom";

import Cookies from "js-cookie";
import axios from "axios";

export async function loader() {
  let accessToken = Cookies.get("accessToken");
  if (accessToken !== undefined) {
    return redirect("/");
  }

  return null;
}

export async function action({ request }) {
  const formData = await request.formData();
  if (formData.get("password") === formData.get("password_confirmation")) {
    const data = JSON.stringify({
      username: formData.get("username"),
      email: formData.get("email"),
      real_name: formData.get("real_name"),
      summary: formData.get("summary"),
      password: formData.get("password"),
    });
    try {
      let res = await axios.post("/users/register", data);
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
  }

  // TODO: place error message in query string
  return redirect("/registrar");
}

export default function Register() {
  return (
    <div className="container vh-100 d-flex justify-content-center align-items-center">
      <Form method="post" className="vstack gap-2 my-auto">
        <h3>Registrar-se no Microblogs</h3>
        <input
          type="text"
          name="username"
          id="username"
          placeholder="Nome da conta"
          className="form-control"
        />
        <input
          type="email"
          name="email"
          id="email"
          placeholder="Email"
          className="form-control"
        />
        <input
          type="text"
          name="real_name"
          id="real_name"
          placeholder="Nome real"
          className="form-control"
        />
        <textarea
          name="summary"
          id="summary"
          placeholder="Breve resumo (Opcional)"
          className="form-control"
        />
        <input
          type="password"
          name="password"
          id="password"
          placeholder="Senha"
          className="form-control"
        />
        <input
          type="password"
          name="password_confirmation"
          id="password_confirmation"
          placeholder="Confirme sua senha"
          className="form-control"
        />
        <button type="submit" className="btn btn-primary">
          Registrar
        </button>
        <p className="text-muted text-center">
          Já possui uma conta? Entre <Link to="/entrar">aqui</Link>.
        </p>
      </Form>
    </div>
  );
}
