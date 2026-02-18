const http = require('http');
const chalk = require('chalk');

console.log(chalk.blue(`
             ____ _                 _ 
            / ___| | ___  _   _  __| |
           | |   | |/ _ \\| | | |/ _  |
           | |___| | (_) | |_| | (_| |
            \\____|_|\\___/ \\__,_|\\__,_|
                            
                             
`));
console.log(chalk.cyan('Contact me for Help @Explainable, Also use http://localhost:4311/api/Explainable ("GET") Request\n'));

function generateRandomString(length) {
    const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let result = '';
    for (let i = 0; i < length; i++) {
        result += characters.charAt(Math.floor(Math.random() * characters.length));
    }
    return result;
}

//##Gen The clearance Shit
function generateCfClearance() {
    const part1 = generateRandomString(16);
    const part2 = generateRandomString(20);
    const part3 = Math.floor(Date.now() / 1000);
    const part4 = "1.0.1.1";  // Fixed value
    const part5 = generateRandomString(20);
    const part6 = generateRandomString(80);

    return `${part1}-${part3}-${part4}-${part5}-${part6}`;
}

//## Gen the __cf_bm Key
function generateCfBm() {
    const part1 = generateRandomString(20);
    const part2 = Math.floor(Date.now() / 1000);
    const part3 = "1.0.1.1";
    const part4 = generateRandomString(40);

    return `__cf_bm=${part1}-${part2}-${part3}-${part4}`;
}

//## Start Simple server
const server = http.createServer((req, res) => {
    if (req.method === 'GET' && req.url === '/api/Explainable') {
        const cf_clearance = generateCfClearance();
        const cf_bm = generateCfBm();

        const responseData = {
            cf_clearance: cf_clearance,
            cf_bm: cf_bm,
        };

        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(responseData));

        console.log(chalk.green(`cf_clearance <cf_clearance>: ${cf_clearance}`));
        console.log(chalk.green(`cf_bm <cf_bm>: ${cf_bm}\n`));
    } else {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('Not Found');
    }
});

//## Used Port 4311 

server.listen(4311, () => {
    console.log(chalk.yellow('For an (update) if needed lmk. ') + chalk.green('When making request parse the 2 keys without placeholders, then at end of your get or post url you will add ?__cf_bm=<cf_bmKey> & also in cookie field add Set-Cookie: cf_clearance=<Clearanceparsedname>'));
});
