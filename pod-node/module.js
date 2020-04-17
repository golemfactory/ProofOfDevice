const addon = require('./build/Release/module');

const USERNAME = "some_username";
const SPID = "2CFD6C88BD1B280D3D6C33977D53A1A3"

let quote = addon.register(SPID, USERNAME);
let message = quote.toString('base64');
console.log(message);
