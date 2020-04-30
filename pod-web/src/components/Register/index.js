import React, { useEffect, useState } from "react";

import { form, register } from "./register.module.css";
import useEventListener from "../../hooks/useEventListener";
import {HOST, REGISTER, REMOTE, ORIGIN} from "../../Constants";

const getById = id => document.getElementById(id);

const Register = () => {
	const [ quote, setQuote ] = useState(null);
	useEffect(() => {
		window.postMessage(
			{ host: HOST, type: REGISTER },
			ORIGIN
		);
	}, []);

	const messageListener = ({data, origin}) => {
		if (origin !== ORIGIN || data.host !== REMOTE)
			return;
		console.log(data); // Answer here
		setQuote(data.data)
	};

	useEventListener("message", messageListener);

	const requestRegister = (event) => {
		event.preventDefault();
		const login = getById("usernameInput").value;
		//const password = getById("passwordInput").value;
		registerRequest({ login, quote })
			.then((result) => {
				console.log(result);
			})
			.catch((error) => {
				console.warn(error);
			});
	};

	return (
		<div className={register}>
			<form className={form} onSubmit={requestRegister}>
				<fieldset>
					<legend>Registration</legend>
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
						<input
							id="confirmationInput"
							type="password"
							name="confirmation"
							placeholder="Confirmation"
						/>
					</div>
					<div>
						<button type="submit">Register</button>
					</div>
				</fieldset>
			</form>
		</div>
	);
};

export default Register;

function registerRequest({ login, quote }) {
	console.log("request", login, quote);
	return fetch('/register', {
		method: 'post',
		headers: {'Content-Type': 'application/json'},
		body: JSON.stringify({
			login,
			quote,
		})
	}).then(response => response.json())
}
