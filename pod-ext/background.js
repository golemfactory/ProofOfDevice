const { CHALLENGE, REGISTER, SPID } = constants;
var port = browser.runtime.connectNative("pod_app");
console.log('port', port);
let waitingResponse = null;

browser.runtime.onMessage.addListener((request, sender, sendResponse) => {
	if (request.type === REGISTER) {
		console.info(REGISTER);
		port.postMessage({
			msg: REGISTER,
			spid: SPID
		})
		waitingResponse = curryResponse(sendResponse, REGISTER);
	} else if (request.type === CHALLENGE) {
		console.info(CHALLENGE, request.data);
		port.postMessage({
			msg: CHALLENGE,
			challenge: request.data
		})
		waitingResponse = curryResponse(sendResponse, CHALLENGE)
	}
	return true; // for async response
});

port.onMessage.addListener(response => {
	if (response.msg === REGISTER) {
		console.info(REGISTER, response);
		waitingResponse(response.quote)
	} else if (response.msg === CHALLENGE) {
		console.info(CHALLENGE, response);
		waitingResponse(response.signed)
	}
	waitingResponse = null
})

function curryResponse(sendResponse, type) {
	return response => sendResponse({type, response})
}
