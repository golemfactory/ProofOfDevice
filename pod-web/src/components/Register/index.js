import React, { useEffect } from "react";

import { form, register } from "./register.module.css";
import useEventListener from "../../hooks/useEventListener";
import {HOST, REGISTER, REMOTE, ORIGIN} from "../../Constants";

const getById = (id) => document.getElementById(id);

const messageListener = (evt) => {
	if (evt.origin !== ORIGIN || evt.data.host !== REMOTE)
		return;
	console.log(evt.data); // Answer here
};

const Register = () => {
	useEffect(() => {
		window.postMessage(
			{ host: HOST, type: REGISTER },
			ORIGIN
		);
	}, []);

	useEventListener("message", messageListener);

	const requestRegister = (event) => {
		event.preventDefault();
		const username = getById("usernameInput").value;
		const password = getById("passwordInput").value;
		mockRequest({ username, password })
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

/*mock*/
const users = ["mdt", "kubkon", "lukasz"];
function mockRequest({ username, password }) {
	return new Promise((resolve, reject) => {
		setTimeout(() => {
			if (users.includes(username)) {
				reject({ status: 403, error: true, message: "User already exist" });
			} else {
				resolve({ status: 200, error: false, message: "Registered" });
			}
		}, 1000);
	});
}
