import CDP from "chrome-remote-interface";
import { launchChrome, saveCookies } from "./chrome";

const TARGET_URL = "https://bigmodel.cn/usercenter/glm-coding/usage";
const CDP_PORT = 9222;
const PROFILE_NAME = "zhipu";

export async function fetchZhipuCookies(): Promise<void> {
  const outputDir = process.env.COOKIES_OUTPUT_PATH || process.cwd();
  let client: Awaited<ReturnType<typeof CDP>> | null = null;
  let cleanupChrome: (() => void) | null = null;

  try {
    cleanupChrome = await launchChrome({ port: CDP_PORT, profileName: PROFILE_NAME });

    console.log("🔌 连接到 Chrome DevTools Protocol...");
    client = await CDP({ port: CDP_PORT });

    const { Network, Page, Runtime } = client;

    await Network.enable();
    await Page.enable();
    await Runtime.enable();

    console.log("📄 打开智谱 coding usage 页面...");
    await Page.navigate({ url: TARGET_URL });

    console.log("⏳ 等待页面加载...");
    await Page.loadEventFired();

    console.log("\n" + "=".repeat(60));
    console.log("✅ 浏览器已打开页面，请在浏览器中完成登录");
    console.log("🔍 登录成功后，脚本会自动检测并获取 Cookies");
    console.log("=".repeat(60) + "\n");

    let isLoggedIn = false;
    let attempts = 0;
    const maxAttempts = 300;

    while (!isLoggedIn && attempts < maxAttempts) {
      attempts++;

      try {
        const urlResult = await Runtime.evaluate({
          expression: "window.location.href",
          returnByValue: true,
        });

        const currentUrl = urlResult.result.value;
        console.log(`[${attempts}] 当前页面: ${currentUrl}`);

        if (currentUrl.includes("glm-coding/usage")) {
          const userCheck = await Runtime.evaluate({
            expression: `
              document.querySelector('[class*="user"]') ||
              document.querySelector('[class*="avatar"]') ||
              document.cookie.includes('authorization')
            `,
            returnByValue: true,
          });

          if (userCheck.result.value) {
            isLoggedIn = true;
            break;
          }
        }

        if (currentUrl.includes("login")) {
          console.log("⚠️  检测到登录页面，等待用户完成登录...");
        }
      } catch {
        // 页面可能还在加载，忽略错误继续轮询
      }

      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    if (!isLoggedIn) {
      console.log("\n❌ 登录检测超时，请重试");
      process.exit(1);
    }

    console.log("\n✅ 检测到登录成功！");
    console.log("🍪 正在获取 Cookies...\n");

    const cookiesResult = await Network.getCookies();
    const relevantCookies = cookiesResult.cookies.filter(
      (cookie) => cookie.domain.includes("bigmodel") || cookie.domain.includes("zhipuai")
    );

    await saveCookies(relevantCookies, outputDir);
  } catch (error) {
    console.error("❌ 发生错误:", error);
    process.exit(1);
  } finally {
    if (client) {
      await client.close();
    }
    if (cleanupChrome) {
      console.log("🧹 关闭 Chrome 浏览器...");
      cleanupChrome();
    }
  }
}
