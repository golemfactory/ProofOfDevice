import React, { useContext } from "react";
import { BrowserRouter as Router, Redirect, Switch, Route } from "react-router-dom";
import AuthContext from "./context/auth";
import Header from "./components/Header";
import Home from "./components/Home";
import Login from "./components/Login";
import Register from "./components/Register";

function RootRouter() {
  const auth = useContext(AuthContext);
  return (
    <Router>
      <div>
        <Header/>
        <Switch>
          <Route path="/Login">
            {!auth.isLoggedIn ? <Login /> : <Redirect to="/" />}
          </Route>
          <Route path="/Register">
            {!auth.isLoggedIn ? <Register /> : <Redirect to="/" />}
          </Route>
          <Route exact path="/">
            {auth.isLoggedIn ? <Home /> : <Redirect to="/login" />}
          </Route>
        </Switch>
      </div>
    </Router>
  );
}

export default RootRouter;
