import React, { Component } from "react";
import AuthContext from "../context/auth";
import Cookies from 'js-cookie';

class AuthProvider extends Component {
	state = {
		isLoggedIn: checkCookie(),
	};

	// make get request to `/` here if login is successful or not
	setLoggedIn = (isLoggedIn) => {
		if(!isLoggedIn) removeCookies();
		this.setState({isLoggedIn});
	}

	render() {
		return (
			<AuthContext.Provider
				value={{ ...this.state, setLoggedIn: this.setLoggedIn }}
			>
				{this.props.children}
			</AuthContext.Provider>
		);
	}
}

export default AuthProvider;

function checkCookie() { // mock
	const value = Cookies.get('auth');
	console.log(value);
	return !!value;
}

function removeCookies() {
	Cookies.remove('session');
}