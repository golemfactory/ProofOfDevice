import React, { Fragment, useContext } from "react";
import AuthContext from "../../context/auth";
import { NavLink } from "react-router-dom";

import { appHeader, brand, navActive, navItem } from "./header.module.css";

const Header = () => {
	const auth = useContext(AuthContext);

	const logout = () => auth.setLoggedIn(false);

	return (
		<header className={appHeader}>
			<div className={brand}>The X Bank</div>
			<nav>
				{auth.isLoggedIn ? (
					<div className={navItem}>
						<NavLink exact={true} activeClassName={navActive} to="/">
							Home
						</NavLink>
					</div>
				) : null}
				{!auth.isLoggedIn ? (
					<Fragment>
						<div className={navItem}>
							<NavLink activeClassName={navActive} to="/login">
								Login
							</NavLink>
						</div>
						<div className={navItem}>
							<NavLink activeClassName={navActive} to="/register">
								Register
							</NavLink>
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
