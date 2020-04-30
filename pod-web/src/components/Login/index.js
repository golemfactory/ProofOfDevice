import React, { useContext, useEffect, useState } from "react";
import AuthContext from "../../context/auth";
import useEventListener from "../../hooks/useEventListener";
import { CHALLENGE, HOST, REMOTE, ORIGIN } from "../../Constants";

import { form, login } from "./login.module.css";

const getById = id => document.getElementById(id);

const Login = () => {
	const auth = useContext(AuthContext);
	const [ signed, setSigned ] = useState(null);
	// const [loading, setLoading] = useState(false) for loading animation
	useEffect(() => {
		challengeRequest()
			.then(({challenge}) => {
				if(!challenge) throw new Error("Challenge cannot be empty.\nHint: Try removing cookies.");
				window.postMessage(
					{ host: HOST, type: CHALLENGE, data: challenge },
					ORIGIN
				);
			})
			.catch((error) => {
				console.warn(error);
			});
	}, []);

	const messageListener = ({data, origin}) => {
		if (origin !== ORIGIN || data.host !== REMOTE) return;
		console.log(data); // Answer
		setSigned(data.data);
	};

	useEventListener("message", messageListener);

	const requestLogin = (event) => {
		event.preventDefault();
		const login = getById("usernameInput").value;
		//const password = getById("passwordInput").value;
		authRequest({ login, signed_challenge: signed })
			.then((result) => {
				console.log(result);
				if(result.status === "Ok") {
					auth.setLoggedIn(true);
				} else {
					alert(result.error);
				}
			})
			.catch((error) => {
				console.warn(error);
			});
	};

	return (
		<div className={login}>
			<form className={form} onSubmit={requestLogin}>
				<fieldset>
					<legend>Login</legend>
					<div>
						<input
							id="usernameInput"
							type="text"
							name="text"
							placeholder="Username"
						/>
					</div>
					<div>
						<input
							id="passwordInput"
							type="password"
							name="password"
							placeholder="Pasword"
						/>
					</div>
					<div>
						<button type="submit">Login</button>
					</div>
				</fieldset>
			</form>
		</div>
	);
};

export default Login;

function challengeRequest() {
	return fetch('/auth', {
		method: 'get'
	}).then(response => response.json())
}

function authRequest({ login, signed_challenge }) {
	console.log('params', login, signed_challenge)
	return fetch('/auth', {
		method: 'post',
		headers: {'Content-Type': 'application/json'},
		body: JSON.stringify({
			login,
			response: signed_challenge,
		})
	}).then(response => response.json())
}
