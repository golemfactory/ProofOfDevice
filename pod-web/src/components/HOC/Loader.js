import React, { useEffect, useState } from "react";
import { tween } from "shifty";

import { loaderIcon, loaderScreen } from "./loader.module.css";
import loadingSVG from "../../assets/dollar.svg";

let time;
function Loader({ children, loadingTime }) {
	const [load, setLoad] = useState(false);
	const [anim, setAnim] = useState([]);
	useEffect(() => {
		time = setTimeout(() => setLoad(true), loadingTime);

		tween({
			from: { z: 0 },
			to: { z: -30 },
			duration: 500,
			step: ({ z, y }) =>
				setAnim([
					{ transform: `rotateZ(${z}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 2}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 3}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 4}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 5}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 6}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 7}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 8}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 9}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 10}deg)`, transformOrigin: 'left' },
					{ transform: `rotateZ(${z * 11}deg)`, transformOrigin: 'left' }
				])
		}, [loadingTime]);

		return () => {
			clearTimeout(time);
			time = null;
		};
	}, [loadingTime]);
	if (load) return children;
	return (
		<div className={loaderScreen}>
			<img src={loadingSVG} className={loaderIcon} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[0]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[1]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[2]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[3]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[4]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[5]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[6]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[7]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[8]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[9]} alt="loading-icon"/>
			<img src={loadingSVG} className={loaderIcon} style={anim[10]} alt="loading-icon"/>
		</div>
	);
}
export default Loader;
