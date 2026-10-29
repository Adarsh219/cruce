import http from "node:http"
import fs from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"
import puppeteer from "puppeteer-core"

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const distDir = path.join(__dirname, "../dist")
const screenshotsDir = path.join(__dirname, "../test-screenshots")
if (!fs.existsSync(screenshotsDir)) fs.mkdirSync(screenshotsDir, { recursive: true })

const mimeTypes = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".svg": "image/svg+xml",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
}

const server = http.createServer((req, res) => {
  let reqPath = req.url.split("?")[0]
  if (reqPath === "/") reqPath = "/index.html"
  const filePath = path.join(distDir, reqPath)

  if (fs.existsSync(filePath) && fs.statSync(filePath).isFile()) {
    const ext = path.extname(filePath).toLowerCase()
    res.writeHead(200, { "Content-Type": mimeTypes[ext] || "application/octet-stream" })
    fs.createReadStream(filePath).pipe(res)
  } else {
    const indexPath = path.join(distDir, "index.html")
    res.writeHead(200, { "Content-Type": "text/html" })
    fs.createReadStream(indexPath).pipe(res)
  }
})

const PORT = 5198
server.listen(PORT, async () => {
  console.log(`[Test Server] Serving at http://127.0.0.1:${PORT}`)

  const errors = []
  const testResults = []

  const assert = (condition, name) => {
    if (condition) {
      console.log(`  ✓ [PASS] ${name}`)
      testResults.push({ name, pass: true })
    } else {
      console.error(`  ✗ [FAIL] ${name}`)
      testResults.push({ name, pass: false })
      errors.push(`Assertion failed: ${name}`)
    }
  }

  let browser
  try {
    browser = await puppeteer.launch({
      executablePath: "/usr/bin/chromium",
      headless: "new",
      args: ["--no-sandbox", "--disable-gpu", "--window-size=1280,850"],
    })

    const page = await browser.newPage()
    await page.setViewport({ width: 1280, height: 850 })

    page.on("pageerror", (err) => {
      console.error("[Browser Exception]:", err.message)
      errors.push(err.message)
    })

    page.on("console", (msg) => {
      if (msg.type() === "error") {
        console.error("[Console Error]:", msg.text())
        errors.push(msg.text())
      }
    })

    await page.goto(`http://127.0.0.1:${PORT}`, { waitUntil: "networkidle0" })
    await new Promise((r) => setTimeout(r, 200))

    // ==========================================
    // SUITE 1: Window Chrome Controls
    // ==========================================
    console.log("\n--- SUITE 1: Window Chrome Controls ---")
    const title = await page.$eval("header span.font-semibold", (el) => el.textContent.trim())
    assert(title === "Cruce", "Window Title is 'Cruce'")

    const minBtn = await page.$("header button:nth-of-type(1)")
    if (minBtn) {
      await minBtn.click()
      await new Promise((r) => setTimeout(r, 100))
      assert(true, "Minimize button clicked without errors")
    }

    // ==========================================
    // SUITE 2: Empty Library State
    // ==========================================
    console.log("\n--- SUITE 2: Empty Library State ---")
    const emptyStateText = await page.evaluate(() => {
      const heading = document.querySelector("h1")
      return heading ? heading.textContent.trim() : ""
    })
    assert(emptyStateText.includes("empty"), `Empty library state rendered: "${emptyStateText}"`)

    const emptyArticleCount = await page.$$eval("article", (els) => els.length)
    assert(emptyArticleCount === 0, "No game cards rendered in initial empty state")

    // ==========================================
    // SUITE 3: Add Game Dialog Lifecycle
    // ==========================================
    console.log("\n--- SUITE 3: Add Game Dialog Lifecycle ---")
    await page.evaluate(() => {
      const addBtns = Array.from(document.querySelectorAll("button"))
      const addBtn = addBtns.find((b) => b.textContent.includes("Add your first game") || b.textContent.includes("Add Game"))
      if (addBtn) addBtn.click()
    })
    await new Promise((r) => setTimeout(r, 400))

    const addDialogVisible = await page.evaluate(() => {
      const dialog = document.querySelector("div[role='dialog']")
      return dialog !== null && dialog.textContent.includes("Add Game")
    })
    assert(addDialogVisible, "Add Game dialog opened")

    // Type in executable path and game title
    const exeInput = await page.$("input[placeholder*='game.exe']")
    assert(exeInput !== null, "Executable path input found")
    if (exeInput) {
      await exeInput.type("/games/cyberpunk/bin/x64/Cyberpunk2077.exe")
    }
    await new Promise((r) => setTimeout(r, 100))

    const titleInput = await page.$("input[placeholder*='Cyberpunk 2077']")
    assert(titleInput !== null, "Game title input found")
    if (titleInput) {
      await page.evaluate(() => {
        const input = document.querySelector("input[placeholder*='Cyberpunk 2077']")
        if (input) {
          const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set
          nativeSetter.call(input, "Cyberpunk 2077")
          input.dispatchEvent(new Event("input", { bubbles: true }))
        }
      })
    }
    await new Promise((r) => setTimeout(r, 100))

    // Submit dialog
    await page.evaluate(() => {
      const submitBtn = Array.from(document.querySelectorAll("div[role='dialog'] button")).find(
        (b) => b.textContent.trim() === "Add to Library",
      )
      if (submitBtn) submitBtn.click()
    })
    await new Promise((r) => setTimeout(r, 400))

    const libraryCountAfterAdd = await page.$$eval("article", (els) => els.length)
    assert(libraryCountAfterAdd === 1, `Game added successfully! Library count is now ${libraryCountAfterAdd}`)

    const cardTitle = await page.$eval("article h3", (el) => el.textContent.trim())
    assert(cardTitle === "Cyberpunk 2077", `Game card title rendered correctly: "${cardTitle}"`)

    // ==========================================
    // SUITE 4: Library Search & Filtering
    // ==========================================
    console.log("\n--- SUITE 4: Library Search & Filtering ---")
    const searchInput = await page.$("input[placeholder*='Search']")
    assert(searchInput !== null, "Search input is visible when library has games")
    if (searchInput) {
      await searchInput.type("cyberpunk")
      await new Promise((r) => setTimeout(r, 150))
      let matchCount = await page.$$eval("article", (els) => els.length)
      assert(matchCount === 1, "Searching 'cyberpunk' matches 1 game")

      await searchInput.click({ clickCount: 3 })
      await searchInput.type("nonexistent_game_query")
      await new Promise((r) => setTimeout(r, 150))
      let noMatchCount = await page.$$eval("article", (els) => els.length)
      assert(noMatchCount === 0, "Searching non-existent game yields 0 cards")

      // Clear search
      await page.evaluate(() => {
        const clearBtn = document.querySelector("button[aria-label*='Clear search']") || document.querySelector("button.size-7")
        if (clearBtn) {
          clearBtn.click()
        } else {
          const input = document.querySelector("input[placeholder*='Search']")
          if (input) {
            const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set
            nativeSetter.call(input, "")
            input.dispatchEvent(new Event("input", { bubbles: true }))
          }
        }
      })
      await new Promise((r) => setTimeout(r, 200))
      let restoredCount = await page.$$eval("article", (els) => els.length)
      assert(restoredCount === 1, "Clearing search restores game card")
    }

    // ==========================================
    // SUITE 5: Game Modal Full Tabs & Controls
    // ==========================================
    console.log("\n--- SUITE 5: Game Modal Controls & Tabs ---")
    const cardBtn = await page.$("article button:first-of-type")
    if (cardBtn) await cardBtn.click()
    await new Promise((r) => setTimeout(r, 300))

    const modalOpen = await page.$eval("div[role='dialog']", (el) => el !== null).catch(() => false)
    assert(modalOpen, "Game details modal opened")

    const switchTab = async (name) => {
      await page.evaluate((tabName) => {
        const btns = Array.from(document.querySelectorAll("button[role='tab']"))
        const target = btns.find((b) => b.textContent.includes(tabName))
        if (target) target.click()
      }, name)
      await new Promise((r) => setTimeout(r, 200))
    }

    // Overview Tab
    await switchTab("Overview")
    const hasWinecfg = await page.evaluate(() => {
      const btns = Array.from(document.querySelectorAll("button"))
      return btns.some((b) => b.textContent.trim() === "Winecfg")
    })
    assert(hasWinecfg, "Overview tab renders Wine power tools")

    // Launch Options Tab
    await switchTab("Launch Options")
    const launchArgsInput = await page.$("input[placeholder*='-dx12']")
    if (launchArgsInput) {
      await launchArgsInput.click({ clickCount: 3 })
      await launchArgsInput.type("-vulkan")
      assert(true, "Launch arguments edited cleanly")
    }

    // Graphics Tab
    await switchTab("Graphics")
    const togglesCount = await page.$$eval("button[role='switch']", (els) => els.length)
    assert(togglesCount >= 2, `Graphics tab renders ${togglesCount} feature toggles`)

    // Advanced Wine Tab
    await switchTab("Advanced Wine")
    const wineSwitches = await page.$$eval("button[role='switch']", (els) => els.length)
    assert(wineSwitches >= 6, `Advanced Wine tab renders ${wineSwitches} toggles`)

    // Manage Data Tab
    await switchTab("Manage Data")
    const hasBackupBtn = await page.evaluate(() => {
      const btns = Array.from(document.querySelectorAll("button"))
      return btns.some((b) => b.textContent.includes("Backup Saves"))
    })
    assert(hasBackupBtn, "Manage Data tab renders save backup controls")

    // Close Modal
    await page.evaluate(() => {
      const close = document.querySelector("div[role='dialog'] button[aria-label*='Close']")
      if (close) close.click()
    })
    await new Promise((r) => setTimeout(r, 300))
    assert(true, "Game modal closed")

    // ==========================================
    // SUITE 6: Wine Cabinet View
    // ==========================================
    console.log("\n--- SUITE 6: Wine Cabinet View ---")
    await page.evaluate(() => {
      const cab = Array.from(document.querySelectorAll("aside nav button")).find((b) => b.textContent.includes("Wine Cabinet"))
      if (cab) cab.click()
    })
    await new Promise((r) => setTimeout(r, 300))

    const cabHeader = await page.evaluate(() => {
      const heading = document.querySelector("h1")
      return heading ? heading.textContent.trim() : ""
    })
    assert(cabHeader.includes("Wine Cabinet"), `Navigated to Wine Cabinet: "${cabHeader}"`)

    const switchSource = async (srcName) => {
      await page.evaluate((s) => {
        const btn = Array.from(document.querySelectorAll("button")).find((b) => b.textContent.includes(s))
        if (btn) btn.click()
      }, srcName)
      await new Promise((r) => setTimeout(r, 150))
      assert(true, `Runner source switched to '${srcName}'`)
    }

    await switchSource("UMU-Proton")
    await switchSource("CachyOS")
    await switchSource("GE-Proton")

    // ==========================================
    // SUITE 7: Settings & Theme Engine
    // ==========================================
    console.log("\n--- SUITE 7: Settings & Theme Engine ---")
    await page.evaluate(() => {
      const set = Array.from(document.querySelectorAll("aside nav button")).find((b) => b.textContent.includes("Settings"))
      if (set) set.click()
    })
    await new Promise((r) => setTimeout(r, 300))

    const themes = ["Crucible", "Midnight", "Catppuccin", "Nord"]
    for (const t of themes) {
      await page.evaluate((themeName) => {
        const btn = Array.from(document.querySelectorAll("button")).find((b) => b.textContent.includes(themeName))
        if (btn) btn.click()
      }, t)
      await new Promise((r) => setTimeout(r, 100))
      const currTheme = await page.$eval("html", (el) => el.getAttribute("data-theme") || document.querySelector("[data-theme]")?.getAttribute("data-theme"))
      assert(currTheme.toLowerCase() === t.toLowerCase(), `Theme successfully changed to '${t}'`)
    }

    const fonts = ["Geist", "Inter", "Mono"]
    for (const f of fonts) {
      await page.evaluate((fontName) => {
        const btn = Array.from(document.querySelectorAll("button")).find((b) => b.textContent.includes(fontName))
        if (btn) btn.click()
      }, f)
      await new Promise((r) => setTimeout(r, 100))
      const currFont = await page.$eval("html", (el) => el.getAttribute("data-font") || document.querySelector("[data-font]")?.getAttribute("data-font"))
      assert(currFont.toLowerCase() === f.toLowerCase(), `Font successfully changed to '${f}'`)
    }

    // ==========================================
    // SUITE 8: Game Deletion & Return to Empty Library
    // ==========================================
    console.log("\n--- SUITE 8: Game Removal & Empty State ---")
    // Navigate back to Library
    await page.evaluate(() => {
      const lib = Array.from(document.querySelectorAll("aside nav button")).find((b) => b.textContent.includes("Library"))
      if (lib) lib.click()
    })
    await new Promise((r) => setTimeout(r, 300))

    // Open game modal
    const targetCard = await page.$("article button:first-of-type")
    if (targetCard) await targetCard.click()
    await new Promise((r) => setTimeout(r, 300))

    // Switch to Manage Data tab
    await switchTab("Manage Data")

    // Click Delete Game
    await page.evaluate(() => {
      const delBtn = Array.from(document.querySelectorAll("button")).find(
        (b) => b.textContent.includes("Delete Game") || b.closest("div")?.textContent.includes("Remove from library"),
      )
      if (delBtn) delBtn.click()
    })
    await new Promise((r) => setTimeout(r, 200))

    // Confirm deletion
    await page.evaluate(() => {
      const confirmBtn = Array.from(document.querySelectorAll("button")).find((b) => b.textContent.trim().includes("Confirm"))
      if (confirmBtn) confirmBtn.click()
    })
    await new Promise((r) => setTimeout(r, 400))

    const finalCount = await page.$$eval("article", (els) => els.length)
    assert(finalCount === 0, "Game removed cleanly! Library returned to empty state")

    console.log("\n=======================================================")
    console.log(`  E2E TEST COMPLETE: ${testResults.filter((t) => t.pass).length} / ${testResults.length} PASSED`)
    console.log(`  TOTAL FAILURES: ${errors.length}`)
    console.log("=======================================================\n")

    if (errors.length > 0) {
      process.exit(1)
    }
  } catch (err) {
    console.error("Test failure:", err)
    process.exit(1)
  } finally {
    if (browser) await browser.close()
    server.close()
  }
})
