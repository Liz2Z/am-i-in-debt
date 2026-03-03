import { fetchZhipuCookies } from "./zhipu";
import { fetchKimiCookies } from "./kimi";

const args = process.argv.slice(2);
const platform = args[0] || "zhipu";

async function main() {
  console.log(`\n🚀 启动 ${platform} 登录流程...\n`);

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
