import React, { useEffect, useState } from "react";
import { podImg, podIndicator, podInfo } from "./pod.module.css";

import podEnabledAsset from "../../assets/pod-enabled.svg";
import podDisabledAsset from "../../assets/pod-disabled.svg";
import podWaitingAsset from "../../assets/pod-waiting.svg";

const PodIndicator = ({ isReady = false }) => {
	const [isEnabled, setEnabled] = useState(false);
	useEffect(() => {
		if (window.__PROOF_OF_DEVICE__) {
			setEnabled(true);
		}
	}, []);

	return (
		<div className={podIndicator}>
			<img
				src={
					isEnabled
						? isReady
							? podEnabledAsset
							: podWaitingAsset
						: podDisabledAsset
				}
				className={podImg}
				alt="pod"
			/>
			<span className={podInfo}>
				{isEnabled
					? isReady
						? "Proof of Device has been enabled."
						: "Waiting reponse from PoD extension"
					: "Proof of Device is not available."}
			</span>
		</div>
	);
};

export default PodIndicator;
