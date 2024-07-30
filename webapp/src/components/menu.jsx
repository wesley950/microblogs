import { Link, useRouteLoaderData } from "react-router-dom";
import UserAvatar from "./user-avatar";

export default function Menu() {
  const { username, realName } = useRouteLoaderData("root");

  return (
    <div
      className="vstack gap-2 mt-2 sticky-sm-top ms-0 ms-sm-auto"
      style={{
        maxWidth: "200px",
      }}
    >
      <Link to="/" className="text-decoration-none fs-5">
        <i className="bi bi-house-fill"></i> início
      </Link>
      <Link to="/pesquisar" className="text-decoration-none fs-5">
        <i className="bi bi-binoculars-fill"></i> pesquisar
      </Link>
      <Link to="/notificações" className="text-decoration-none fs-5">
        <i className="bi bi-chat-fill"></i> notificações
      </Link>
      <Link to="/configurações" className="text-decoration-none fs-5">
        <i className="bi bi-gear-fill"></i> configurações
      </Link>

      <UserAvatar username={username} realName={realName} linkToProfile />
    </div>
  );
}
