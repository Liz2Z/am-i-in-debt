import { fetchZhipuCookies } from "./zhipu";
import { fetchKimiCookies } from "./kimi";
import { initScriptLogger } from "./logger";

const args = process.argv.slice(2);
const platform = args[0] || "zhipu";

async function main() {
  const outputDir = process.env.COOKIES_OUTPUT_PATH || process.cwd();
  const logPath = initScriptLogger(platform, outputDir);

  console.log(`\n🚀 启动 ${platform} 登录流程...\n`);
  console.log(`📝 登录脚本日志: ${logPath}`);

  switch (platform) {
    case "zhipu":
      await fetchZhipuCookies();
      break;
    case "kimi":
      await fetchKimiCookies();
      break;
    default:
      console.error(`❌ 未知的平台: ${platform}`);
      console.log("用法: get-cookies <zhipu|kimi>");
      process.exit(1);
  }
}

main();
