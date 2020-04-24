import React from "react";
import RootRouter from "./Router";
import AuthProvider from "./provider/authProvider";
import "./App.css";

function App() {
	return (
		<AuthProvider>
			<div className="App">
				<RootRouter />
			</div>
		</AuthProvider>
	);
}

export default App;
