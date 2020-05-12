import React, { useEffect, useState } from "react";

import { form, register } from "./register.module.css";
import useEventListener from "../../hooks/useEventListener";
import request from "../../utils/request";
import {HOST, REGISTER, REMOTE, ORIGIN} from "../../Constants";
import PodIndicator from "../PodIndicator";

const getById = id => document.getElementById(id);

const Register = () => {
	const [ errorStatus, setError ] = useState(false);
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
				setError(true)
			});
	};

	return (
		<div className={register}>
			<form className={form} onSubmit={requestRegister} autoComplete="off">
				<fieldset>
					<legend>Registration</legend>
					{errorStatus ? <div>User already exist!</div> : null}
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
							autoComplete="new-password"
						/>
					</div>
					<div>
						<input
							id="confirmationInput"
							type="password"
							name="confirmation"
							placeholder="Confirmation"
							autoComplete="new-password"
						/>
					</div>
					<div>
						<button type="submit">Register</button>
					</div>
					<PodIndicator isReady={!!quote} />
				</fieldset>
			</form>
		</div>
	);
};

export default Register;

function registerRequest({ login, quote }) {
	return request('/register', {
		method: 'post',
		headers: {'Content-Type': 'application/json'},
		body: JSON.stringify({
			login,
			quote,
		})
	})
}
