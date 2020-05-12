const { HOST, ORIGIN, REMOTE, REGISTER, CHALLENGE } = constants;
var port = browser.runtime.connect();

//insert global variable to be shown
const container = document.head || document.documentElement;
const scriptTag = document.createElement("script");
scriptTag.setAttribute("async", "false");
scriptTag.textContent = "window.__PROOF_OF_DEVICE__ = true;";
container.insertBefore(scriptTag, container.children[0]);
container.removeChild(scriptTag);

window.addEventListener(
	"message",
	function(event) {
		//block unkown origins
		if (event.origin !== ORIGIN || event.data.host !== REMOTE) return;
		if (event.source != window) return;

		if (event.data.type) {
			let messageToBack = browser.runtime.sendMessage({
				type: event.data.type,
				data: event.data.data,
			});
			messageToBack.then(handleResponse.bind(null, event.source), handleError);
		}
	},
	false
);

function handleResponse(source, { type, response }) {
	source.postMessage({ host: HOST, data: `${response}` }, "*");
}

function handleError(error) {
	console.log(`Error: ${error}`);
}
