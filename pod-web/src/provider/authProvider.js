import React, { Component } from "react";
import AuthContext from "../context/auth";

class AuthProvider extends Component {
	state = {
		isLoggedIn: false,
	};

	setLoggedIn = (isLoggedIn) => this.setState({isLoggedIn});

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
