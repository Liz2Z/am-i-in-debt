/**
 * Chrome DevTools Protocol 获取 Kimi Cookies
 *
 * 自动启动 Chrome 浏览器并获取 cookies
 * 使用环境变量 COOKIES_OUTPUT_PATH 指定输出目录
 */

import CDP from "chrome-remote-interface";
import { spawn } from "child_process";

const TARGET_URL = "https://www.kimi.com/code/console";
const CDP_PORT = 9223;

// 获取输出目录（从环境变量或当前目录）
const OUTPUT_DIR = process.env.COOKIES_OUTPUT_PATH || process.cwd();

// 启动 Chrome 浏览器
function launchChrome(): Promise<() => void> {
  return new Promise((resolve, reject) => {
    const isMac = process.platform === "darwin";
    const isWindows = process.platform === "win32";
    const isLinux = process.platform === "linux";

    // 先清理可能占用端口的进程
    console.log(`🧹 清理端口 ${CDP_PORT}...`);

    if (isMac || isLinux) {
      try {
        // 查找占用端口的进程并杀死
        const { execSync } = require("child_process");
        try {
          const pid = execSync(`lsof -ti:${CDP_PORT}`, {
            encoding: "utf-8",
          }).trim();
          if (pid) {
            console.log(`🔪 杀死占用端口的进程: ${pid}`);
            execSync(`kill -9 ${pid}`);
          }
        } catch (e) {
          // 没有进程占用端口，忽略
        }
      } catch (e) {
        console.log("清理端口时出错，继续...");
      }
    }

    let chromePath: string;
    let args: string[];

    // 使用临时用户数据目录，避免与现有 Chrome 冲突
    const tmpDir = process.env.TMPDIR || "/tmp";
    const userDataDir = `${tmpDir}/chrome-debug-kimi-${Date.now()}`;

    if (isMac) {
      chromePath =
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
        "--disable-background-networking",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--disable-features=TranslateUI",
        "--disable-ipc-flooding-protection",
        "--disable-features=BlinkGenPropertyTrees",
      ];
    } else if (isWindows) {
      chromePath = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
      ];
    } else {
      // Linux
      chromePath = "google-chrome";
      args = [
        `--remote-debugging-port=${CDP_PORT}`,
        "--no-first-run",
        "--no-default-browser-check",
        `--user-data-dir=${userDataDir}`,
        "--no-sandbox",
      ];
    }

    console.log(`🚀 启动 Chrome 浏览器...`);

    const chromeProcess = spawn(chromePath, args, {
      detached: true,
      stdio: "ignore",
    });

    chromeProcess.on("error", (err) => {
      reject(new Error(`启动 Chrome 失败: ${err.message}`));
    });

    // Chrome 启动后分离进程
    chromeProcess.unref();

    // 等待 Chrome 启动并创建可检查的 target
    let retries = 0;
    const maxRetries = 20; // 增加到 20 秒
    const checkInterval = setInterval(async () => {
      retries++;
      try {
        // 尝试连接 CDP 来检查 Chrome 是否准备好
        const testClient = await CDP({ port: CDP_PORT }).catch(() => null);
        if (testClient) {
          await testClient.close();
          clearInterval(checkInterval);
          console.log("✅ Chrome 已准备就绪");
          resolve(() => {
            // 清理函数：关闭这个特定的 Chrome 实例
            try {
              if (isMac || isLinux) {
                spawn("pkill", ["-9", "-f", "chrome-debug-kimi"]);
              } else if (isWindows) {
                spawn("taskkill", ["/F", "/IM", "chrome.exe"]);
              }
            } catch (e) {
              // 忽略关闭错误
            }
          });
        } else if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error("Chrome 启动超时"));
        }
      } catch (e) {
        if (retries >= maxRetries) {
          clearInterval(checkInterval);
          reject(new Error("Chrome 启动超时"));
        }
      }
    }, 1000);
  });
}

// 验证 token 是否有效的函数
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
      },
    );

    const data = await response.json();

    // 检查是否返回 unauthenticated 错误
    if (data.code === "unauthenticated") {
      return false;
    }

    // 如果有 usages 数组，说明 token 有效
    if (data.usages) {
      return true;
    }

    return false;
  } catch (error) {
    return false;
  }
}

async function main() {
  let client: Awaited<ReturnType<typeof CDP>> | null = null;
  let cleanupChrome: (() => void) | null = null;

  try {
    // 启动 Chrome
    cleanupChrome = await launchChrome();

    console.log("🔌 连接到 Chrome DevTools Protocol...");
    client = await CDP({ port: CDP_PORT });

    const { Network, Page } = client;

    // 启用必要的域
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

    // 轮询检查登录状态 - 每 3 秒一次
    let isLoggedIn = false;
    let attempts = 0;
    const maxAttempts = 30; // 最多等待 5 分钟 (30 * 3s = 90s)

    while (!isLoggedIn && attempts < maxAttempts) {
      attempts++;

      try {
        // 获取 kimi.com 域名的所有 cookies（包括 httpOnly）
        const cookies = await Network.getCookies({
          urls: ["https://www.kimi.com/"],
        });
        const kimiAuthCookie = cookies.cookies.find(
          (c: any) => c.name === "kimi-auth",
        );

        // 打印所有找到的 kimi 相关 cookies（调试用）
        const kimiCookies = cookies.cookies.filter(
          (c: any) =>
            c.domain.includes("kimi") || c.domain.includes("moonshot"),
        );
        if (kimiCookies.length > 0) {
          console.log(
            `[${attempts}] 找到 ${kimiCookies.length} 个 kimi 相关 cookies:`,
          );
          kimiCookies.forEach((c: any) => {
            console.log(
              `    - ${c.name} (httpOnly: ${c.httpOnly}, secure: ${c.secure})`,
            );
          });
        }

        if (kimiAuthCookie) {
          const token = kimiAuthCookie.value;
          const tokenPreview = token.substring(0, 20) + "...";

          console.log(
            `[${attempts}] ✅ 检测到 kimi-auth cookie: ${tokenPreview}`,
          );
          console.log(`[${attempts}] 调用 usage 接口验证...`);

          // 验证 token 是否有效
          const isValid = await validateToken(token);

          if (isValid) {
            console.log(`[${attempts}] ✅ usage 接口调用成功！登录成功！`);
            isLoggedIn = true;
            break;
          } else {
            console.log(
              `[${attempts}] ⚠️  usage 接口返回失败，token 无效，继续等待...`,
            );
          }
        } else {
          console.log(
            `[${attempts}] ⚠️  未检测到 kimi-auth cookie，继续等待...`,
          );
        }
      } catch (error) {
        console.log(`[${attempts}] 检查时出错: ${error}`);
      }

      // 每 3 秒检查一次
      await new Promise((resolve) => setTimeout(resolve, 3000));
    }

    if (!isLoggedIn) {
      console.log("\n❌ 登录检测超时，请重试");
      process.exit(1);
    }

    console.log("\n✅ 检测到登录成功！");
    console.log("🍪 正在获取 Cookies...\n");

    // 获取 kimi.com 域名的所有 cookies（包括 httpOnly）
    const cookiesResult = await Network.getCookies({
      urls: ["https://www.kimi.com/"],
    });

    // 筛选 kimi.com 相关的 cookies
    const relevantCookies = cookiesResult.cookies.filter(
      (cookie) =>
        cookie.domain.includes("kimi") || cookie.domain.includes("moonshot"),
    );

    console.log("=".repeat(60));
    console.log("🎉 成功获取 Cookies:");
    console.log("=".repeat(60));

    // 打印每个 cookie
    relevantCookies.forEach((cookie) => {
      console.log(`\n📌 ${cookie.name}`);
      console.log(
        `   值: ${cookie.value.substring(0, 50)}${cookie.value.length > 50 ? "..." : ""}`,
      );
      console.log(`   域名: ${cookie.domain}`);
      console.log(`   路径: ${cookie.path}`);
      console.log(`   安全: ${cookie.secure ? "是" : "否"}`);
      console.log(`   HttpOnly: ${cookie.httpOnly ? "是" : "否"}`);
    });

    // 生成 Cookie 字符串（用于 API 请求）
    const cookieString = relevantCookies
      .map((c) => `${c.name}=${c.value}`)
      .join("; ");

    console.log("\n" + "=".repeat(60));
    console.log("📋 Cookie 字符串 (可直接用于请求头):");
    console.log("=".repeat(60));
    console.log(cookieString);

    // 保存到文件（使用指定输出目录）
    const jsonPath = `${OUTPUT_DIR}/cookies.json`;
    const txtPath = `${OUTPUT_DIR}/cookies.txt`;

    await Bun.write(jsonPath, JSON.stringify(relevantCookies, null, 2));
    console.log(`\n💾 Cookies 已保存到 ${jsonPath}`);

    await Bun.write(txtPath, cookieString);
    console.log(`💾 Cookie 字符串已保存到 ${txtPath}`);
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

main();
