import React, { useContext, useEffect, useState } from "react";
import AuthContext from "../../context/auth";
import useEventListener from "../../hooks/useEventListener";
import request from "../../utils/request";
import { CHALLENGE, HOST, REMOTE, ORIGIN } from "../../Constants";
import PodIndicator from "../PodIndicator";

import { click, form, login, motto } from "./login.module.css";

import clickAsset from "../../assets/click.svg";

const getById = (id) => document.getElementById(id);

const Login = () => {
	const auth = useContext(AuthContext);
	const [signed, setSigned] = useState(null);
	const [errorStatus, setError] = useState(false);
	// const [loading, setLoading] = useState(false) for loading animation
	useEffect(() => {
		challengeRequest()
			.then(({ challenge }) => {
				if (!challenge)
					throw new Error(
						"Challenge cannot be empty.\nHint: Try removing cookies."
					);
				window.postMessage(
					{ host: HOST, type: CHALLENGE, data: challenge },
					ORIGIN
				);
			})
			.catch((error) => {
				console.warn(error);
				alert(`Network Error: ${error.status}`);
			});
	}, []);

	const messageListener = ({ data, origin }) => {
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
				if (result.status === "Ok") {
					auth.setLoggedIn(true);
				} else {
					setError(true);
				}
			})
			.catch((error) => {
				console.warn(error);
				setError(true);
			});
	};

	return (
		<div className={login}>
			<div className={motto}>
				Possibilites.
				<br />
				One Click Away
				<img className={click} src={clickAsset} alt="cursor pointer"/>
			</div>
			<form className={form} onSubmit={requestLogin} autoComplete="off">
				<fieldset>
					<legend>Login</legend>
					{errorStatus ? <div>Wrong credentials</div> : null}
					<div>
						<input
							id="usernameInput"
							type="text"
							name="text"
							placeholder="Username"
							autoComplete="off"
							autoFocus
						/>
					</div>
					<div>
						<input
							id="passwordInput"
							type="password"
							name="password"
							placeholder="Pasword"
							autoComplete="off"
						/>
					</div>
					<div>
						<button type="submit">Login</button>
					</div>
					<PodIndicator isReady={!!signed} />
				</fieldset>
			</form>
		</div>
	);
};

export default Login;

function challengeRequest() {
	return request("/auth", {
		method: "get",
	});
}

function authRequest({ login, signed_challenge }) {
	return request("/auth", {
		method: "post",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({
			login,
			response: signed_challenge,
		}),
	});
}
