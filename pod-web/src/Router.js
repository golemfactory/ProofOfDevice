import React, { useContext } from "react";
import {
  BrowserRouter as Router,
  Redirect,
  Switch,
  Route,
} from "react-router-dom";
import AuthContext from "./context/auth";
import Loader from "./components/HOC/Loader";
import Header from "./components/Header";
import Home from "./components/Home";
import Login from "./components/Login";
import Register from "./components/Register";

function RootRouter() {
  const auth = useContext(AuthContext);
  const loadingTime = 1000;
  return (
    <Router>
      <div>
        <Header />
        <Switch>
          <Route path="/Login">
            {!auth.isLoggedIn ? (
              <Loader loadingTime={loadingTime}>
                <Login />
              </Loader>
            ) : (
              <Redirect to="/" />
            )}
          </Route>
          <Route path="/Register">
            {!auth.isLoggedIn ? (
              <Loader loadingTime={loadingTime}>
                <Register />
              </Loader>
            ) : (
              <Redirect to="/" />
            )}
          </Route>
          <Route exact path="/">
            {auth.isLoggedIn ? (
              <Loader loadingTime={loadingTime}>
                <Home />
              </Loader>
            ) : (
              <Redirect to="/login" />
            )}
          </Route>
        </Switch>
      </div>
    </Router>
  );
}

export default RootRouter;
