import React, { useContext, useEffect } from "react";
import AuthContext from "../../context/auth";
import useEventListener from "../../hooks/useEventListener";
import { CHALLENGE, HOST, REMOTE, ORIGIN } from "../../Constants";

import { form, login } from "./login.module.css";

const getById = (id) => document.getElementById(id);

const messageListener = (evt) => {
	if (evt.origin !== ORIGIN || evt.data.host !== REMOTE) return;
	console.log(evt.data); // Answer
};

const Login = () => {
	const auth = useContext(AuthContext);
	// const [loading, setLoading] = useState(false) for loading animation

	useEffect(() => {
		window.postMessage(
			{ host: HOST, type: CHALLENGE, data: { value: 4 } },
			ORIGIN
		);
	}, []);

	useEventListener("message", messageListener);

	const requestLogin = (event) => {
		event.preventDefault();
		const username = getById("usernameInput").value;
		const password = getById("passwordInput").value;
		mockRequest({ username, password })
			.then((result) => {
				console.log(result);
				auth.setLoggedIn(true);
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

/*mock*/
const users = ["mdt", "kubkon", "lukasz"];
function mockRequest({ username, password }) {
	return new Promise((resolve, reject) => {
		setTimeout(() => {
			if (users.includes(username)) {
				resolve({ status: 200, error: false, message: "Logged In" });
			} else {
				reject({ status: 403, error: true, message: "Wrong information" });
			}
		}, 1000);
	});
}
