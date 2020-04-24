var isFirefox;

function checkBrowser() {
	if (typeof browser === 'undefined') {
		browser = chrome;
	} else {
		isFirefox = true;
	}
	return;
}

checkBrowser();
