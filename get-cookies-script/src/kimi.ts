import CDP from "chrome-remote-interface";
import { launchChrome, saveCookies } from "./chrome";

const TARGET_URL = "https://www.kimi.com/code/console";
const CDP_PORT = 9223;
const PROFILE_NAME = "kimi";

async function validateToken(token: string): Promise<boolean> {
  try {
    const response = await fetch(
      "https://www.kimi.com/apiv2/kimi.gateway.billing.v1.BillingService/GetUsages",
      {
        method: "POST",
        headers: {
          accept: "*/*",
          authorization: `Bearer ${token}`,
          "content-type": "application/json",
          "connect-protocol-version": "1",
          origin: "https://www.kimi.com",
          referer: "https://www.kimi.com/code/console",
          "x-language": "zh-CN",
          "x-msh-platform": "web",
          "x-msh-version": "1.0.0",
          "r-timezone": "Asia/Shanghai",
        },
        body: JSON.stringify({ scope: ["FEATURE_CODING"] }),
      }
    );

    const data = await response.json();

    if (data.code === "unauthenticated") {
      return false;
    }

    return !!data.usages;
  } catch {
    return false;
  }
}

export async function fetchKimiCookies(): Promise<void> {
  const outputDir = process.env.COOKIES_OUTPUT_PATH || process.cwd();
  let client: Awaited<ReturnType<typeof CDP>> | null = null;
  let cleanupChrome: (() => void) | null = null;

  try {
    console.log(`📂 Cookies 输出目录: ${outputDir}`);
    cleanupChrome = await launchChrome({ port: CDP_PORT, profileName: PROFILE_NAME });

    console.log("🔌 连接到 Chrome DevTools Protocol...");
    client = await CDP({ port: CDP_PORT });

    const { Network, Page } = client;

    await Network.enable();
    await Page.enable();

    console.log("📄 打开 Kimi coding console 页面...");
    await Page.navigate({ url: TARGET_URL });

    console.log("⏳ 等待页面加载...");
    await Page.loadEventFired();

    console.log("\n" + "=".repeat(60));
    console.log("✅ 浏览器已打开，请在浏览器中完成登录");
    console.log("🔍 每 3 秒检测一次登录状态...");
    console.log("=".repeat(60) + "\n");

    let isLoggedIn = false;
    let attempts = 0;
    const maxAttempts = 30;

    while (!isLoggedIn && attempts < maxAttempts) {
      attempts++;

      try {
        const cookies = await Network.getCookies({ urls: ["https://www.kimi.com/"] });
        const kimiAuthCookie = cookies.cookies.find((c) => c.name === "kimi-auth");

        const kimiCookies = cookies.cookies.filter(
          (c) => c.domain.includes("kimi") || c.domain.includes("moonshot")
        );

        if (kimiCookies.length > 0) {
          console.log(`[${attempts}] 找到 ${kimiCookies.length} 个 kimi 相关 cookies:`);
          kimiCookies.forEach((c) => {
            console.log(`    - ${c.name} (httpOnly: ${c.httpOnly}, secure: ${c.secure})`);
          });
        }

        if (kimiAuthCookie) {
          const token = kimiAuthCookie.value;
          const tokenPreview = token.substring(0, 20) + "...";

          console.log(`[${attempts}] ✅ 检测到 kimi-auth cookie: ${tokenPreview}`);
          console.log(`[${attempts}] 调用 usage 接口验证...`);

          const isValid = await validateToken(token);

          if (isValid) {
            console.log(`[${attempts}] ✅ usage 接口调用成功！登录成功！`);
            isLoggedIn = true;
            break;
          } else {
            console.log(`[${attempts}] ⚠️  usage 接口返回失败，token 无效，继续等待...`);
          }
        } else {
          console.log(`[${attempts}] ⚠️  未检测到 kimi-auth cookie，继续等待...`);
        }
      } catch (error) {
        console.warn(`[${attempts}] 检查时出错:`, error);
      }

      await new Promise((resolve) => setTimeout(resolve, 3000));
    }

    if (!isLoggedIn) {
      console.log("\n❌ 登录检测超时，请重试");
      process.exit(1);
    }

    console.log("\n✅ 检测到登录成功！");
    console.log("🍪 正在获取 Cookies...\n");

    const cookiesResult = await Network.getCookies({ urls: ["https://www.kimi.com/"] });
    const relevantCookies = cookiesResult.cookies.filter(
      (cookie) => cookie.domain.includes("kimi") || cookie.domain.includes("moonshot")
    );

    await saveCookies(relevantCookies, outputDir);
  } catch (error) {
    console.error("❌ 发生错误:", error);
    process.exit(1);
  } finally {
    console.log("🏁 登录脚本结束，开始资源清理");
    if (client) {
      await client.close();
    }
    if (cleanupChrome) {
      console.log("🧹 关闭 Chrome 浏览器...");
      cleanupChrome();
    }
  }
}
