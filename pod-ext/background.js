const { REGISTER, CHALLENGE } = constants;
var port = browser.runtime.connectNative("pod_app");

browser.runtime.onMessage.addListener((request, sender, sendResponse) => {
	if (request.type === REGISTER) {
		console.info(REGISTER);
		sendResponse({type: REGISTER, response: "QUOTE_GONNA_BE_HERE"});
	} else if (request.type === CHALLENGE) {
		console.info(CHALLENGE, request.data);
		sendResponse({type: CHALLENGE, response: "SIGNED_CHALLENGE_GONNA_BE_HERE"});
	}
});
