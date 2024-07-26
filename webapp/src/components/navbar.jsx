import { Link } from "react-router-dom";

export default function Navbar() {
  return (
    <nav className="navbar navbar-dark sticky-top bg-primary">
      <div className="container d-flex justify-content-center">
        <Link className="navbar-brand text-lowercase fs-3" to="/">
          <img src="vite.svg" alt="Vite" width={30} />
          Microblogs
        </Link>
      </div>
    </nav>
  );
}
