function parseJSON(response) {
	return new Promise((resolve, reject) =>
		response
			.json()
			.then((json) =>
				resolve({
					status: response.status,
					ok: response.ok,
					json,
				})
			)
			.catch((error) => {
				reject({
					status: response.status,
					message: error.message,
				});
			})
	);
}

/**
 * Requests a URL, returning a promise
 *
 * @param  {string} url       The URL we want to request
 * @param  {object} [options] The options we want to pass to "fetch"
 *
 * @return {Promise}           The request promise
 */
export default function request(url, options) {
	return new Promise((resolve, reject) => {
		fetch(url, options)
			.then(parseJSON)
			.then((response) => {
				if (response.ok) {
					return resolve(response.json);
				}
				let error = response.json?.description;
				return reject(error || response.status);
			})
			.catch((error) => {
				reject(error);
			});
	});
}
