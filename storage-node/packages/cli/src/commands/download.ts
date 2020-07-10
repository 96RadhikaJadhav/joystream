import axios from "axios";
import chalk from "chalk"
import fs from "fs";
import {fail, createAndLogAssetUrl} from "./common";

function validateDownloadParameters(url: string, contentId: string, filePath: string) : boolean {
    return url && url !== "" && contentId && contentId !=="" && filePath && filePath !== "";
}

function showDownloadUsage() {
    console.log(chalk.yellow(`
        Invalid parameters for 'download' command.
        Usage:   storage-cli download colossusURL contentID filePath
        Example: storage-cli download http://localhost:3001 0x7a6ba7e9157e5fba190dc146fe1baa8180e29728a5c76779ed99655500cff795 ./movie.mp4
      `));
}

export async function run(api: any, url: string, contentId: string, filePath: string){
    // Create, validate and show parameters.
    if (!validateDownloadParameters(url, contentId, filePath)) {
        return showDownloadUsage();
    }
    const assetUrl = createAndLogAssetUrl(url, contentId);
    console.log(chalk.yellow('File path:', filePath));

    // Create file write stream and set error handler.
    const writer = fs.createWriteStream(filePath)
        .on('error', (err) => {
            fail(`File write failed: ${err}`);
        });

    // Request file download.
    try {
        const response = await axios({
            url: assetUrl,
            method: 'GET',
            responseType: 'stream'
        });

        response.data.pipe(writer);

        return new Promise((resolve) => {
            writer.on('finish', () => {
                console.log("File downloaded.")
                resolve();
            });
        });
    } catch (err) {
        fail(`Colossus request failed: ${err.message}`);
    }
}
