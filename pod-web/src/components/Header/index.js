import React, { Fragment, useContext, useEffect } from "react";
import AuthContext from "../../context/auth";
import { NavLink } from "react-router-dom";

import { appHeader, navActive, navItem } from "./header.module.css";

const Header = () => {
	const auth = useContext(AuthContext);
	useEffect(() => {
		const port = null;
		if (!port) {
			console.log("extension not installed");
		}
	}, []);

	const logout = () => auth.setLoggedIn(false);

	return (
		<header className={appHeader}>
			<nav>
				<div className={navItem}>
					<NavLink exact={true} activeClassName={navActive} to="/">Home</NavLink>
				</div>
				{!auth.isLoggedIn ? (
					<Fragment>
						<div className={navItem}>
							<NavLink activeClassName={navActive} to="/login">Login</NavLink>
						</div>
						<div className={navItem}>
							<NavLink activeClassName={navActive} to="/register">Register</NavLink>
						</div>
					</Fragment>
				) : (
					<div className={navItem}>
						<span onClick={logout}>Logout</span>
					</div>
				)}
			</nav>
		</header>
	);
};

export default Header;
