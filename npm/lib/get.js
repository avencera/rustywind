// @ts-check
"use strict";

const https = require("https");
const url = require("url");
const proxy_from_env = require("proxy-from-env");

module.exports = get;

/**
 * @param {string} _url
 * @param {https.RequestOptions} opts
 * @returns
 */
function get(_url, opts) {
  console.log(`GET ${_url}`);

  const proxy = proxy_from_env.getProxyForUrl(url.parse(_url));
  if (proxy !== "") {
    var HttpsProxyAgent = require("https-proxy-agent");
    opts = {
      ...opts,
      agent: new HttpsProxyAgent(proxy),
    };
  }

  return new Promise((resolve, reject) => {
    let result = "";
    opts = {
      ...url.parse(_url),
      ...opts,
    };
    https.get(opts, (response) => {
      if (response.statusCode !== 200) {
        reject(new Error("Request failed: " + response.statusCode));
      }

      response.on("data", (d) => {
        result += d.toString();
      });

      response.on("end", () => {
        resolve(result);
      });

      response.on("error", (e) => {
        reject(e);
      });
    });
  });
}
