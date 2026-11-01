/* Start Vite first: pnpm dev --host 127.0.0.1
   Override executable with CHROMIUM_PATH, or run pnpm exec playwright install chromium. */
import { chromium } from 'playwright'
import assert from 'node:assert/strict'
import fs from 'node:fs'
import process from 'node:process'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
const __dirname = path.dirname(fileURLToPath(import.meta.url))
const base = process.env.U3_QA_URL || 'http://127.0.0.1:1420'
const out = process.env.U3_QA_OUT || path.resolve(__dirname, '../qa-output')
fs.mkdirSync(out, { recursive: true })
;(async () => {
  const browser = await chromium.launch({
    ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
    args: ['--no-sandbox'],
  })
  const errors = [],
    checks = []
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } })
  page.on('pageerror', (e) => errors.push(e.stack || e.message))
  // The test harness and every feature under test are local. Don't send telemetry in QA.
  await page.route('https://**/*', (r) => r.abort())
  try {
    await page.goto(`${base}/tests/theme-preview.html`)
    await page.locator('.themes-page').waitFor({ timeout: 60000 })
    const ids = await page
      .locator('[data-theme-id]')
      .evaluateAll((nodes) => nodes.map((n) => n.dataset.themeId))
    assert.equal(ids.length, 35)
    for (const id of ids) {
      await page.locator(`[data-theme-id="${id}"]`).click()
      await page.waitForFunction((id) => window.__nativeSettings.theme === id, id)
      await page.waitForFunction((id) => document.documentElement.dataset.theme === id, id)
      await page.locator('.router-view').evaluate((el) => el.scrollTo(0, 0))
      if (!process.env.U3_QA_FAST)
        await page
          .locator('.active-theme-panel')
          .screenshot({ animations: 'disabled', path: path.join(out, `theme-${id}.png`) })
    }
    checks.push('35 theme choices applied, persisted and rendered')
    await page.locator('[data-theme-id="aurora"]').click()
    await page.waitForFunction(() => window.__nativeSettings.theme === 'aurora')
    await page.evaluate(() => {
      window.__failThemeSave = true
    })
    await page.locator('[data-theme-id="neon"]').click()
    await page.locator('.save-notice.error').waitFor()
    assert.equal(await page.locator('html').getAttribute('data-theme'), 'aurora')
    await page.evaluate(() => {
      window.__failThemeSave = false
    })
    checks.push('failed theme write restores the previous appearance')
    const favorite = page
      .locator('.theme-entry')
      .filter({ has: page.locator('[data-theme-id="aurora"]') })
      .locator('.favorite-theme')
    await favorite.click()
    await page.getByRole('button', { name: /Избранные/ }).click()
    assert.equal(await page.locator('.theme-choice').count(), 1)
    await page.goto(`${base}/tests/theme-preview.html`)
    await page.locator('.themes-page').waitFor()
    await page.getByRole('button', { name: /Избранные/ }).click()
    assert.equal(await page.locator('.theme-choice').count(), 1)
    await page.getByRole('button', { name: 'Все', exact: true }).click()
    await page.getByRole('searchbox', { name: 'Поиск тем' }).fill('темный')
    // Search includes both Russian names and translated names/effects.
    await page.getByRole('searchbox', { name: 'Поиск тем' }).fill('зачарованный лес')
    assert.equal(await page.locator('.theme-choice').count(), 1)
    await page.getByRole('searchbox', { name: 'Поиск тем' }).fill('no-such-theme')
    await page.locator('.empty-themes').waitFor()
    await page.getByRole('searchbox', { name: 'Поиск тем' }).fill('')
    checks.push('favorite persistence, filters, search and empty state')
    await page.emulateMedia({ reducedMotion: 'reduce' })
    await page.waitForFunction(() => document.documentElement.dataset.jelly === 'off')
    assert.equal(await page.locator('.app-atmosphere').getAttribute('data-running'), 'false')
    await page.emulateMedia({ reducedMotion: 'no-preference' })
    const effects = page.locator('.effect-toggle-row input[type=checkbox]')
    await effects.uncheck()
    assert.equal(await page.locator('.app-atmosphere').count(), 0)
    assert.equal(await page.locator('.theme-chrome').count(), 0)
    await effects.check()
    checks.push('reduced motion and disabled effects')
    await page.locator('.router-view').evaluate((el) => el.scrollTo(0, 0))
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'themes-desktop.png') })
    await page.setViewportSize({ width: 390, height: 844 })
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'themes-narrow.png') })
    const overflow = await page
      .locator('.router-view')
      .evaluate((el) => el.scrollWidth - el.clientWidth)
    assert.ok(overflow <= 2, `narrow view overflow: ${overflow}`)
    checks.push('390px theme page has no horizontal overflow')
    await page.setViewportSize({ width: 1280, height: 900 })
    // Shared modal: keyboard focus, escape, rapid reopen and restore.
    const create = page.locator('.nav-container .settings button.sleek-primary')
    await create.click()
    const dialog = page.getByRole('dialog', { name: /Создать профиль|Create instance/ })
    await dialog.waitFor()
    assert.ok((await dialog.locator('.multiselect__tags').first().boundingBox()).height >= 40)
    for (let i = 0; i < 25; i++) {
      await page.keyboard.press(i % 2 ? 'Shift+Tab' : 'Tab')
      assert.ok(await dialog.evaluate((el) => el.contains(document.activeElement)))
    }
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'modal-desktop.png') })
    await page.keyboard.press('Escape')
    await create.click()
    await dialog.waitFor()
    await page.keyboard.press('Escape')
    await page.waitForTimeout(350)
    assert.equal(await page.getByRole('dialog').count(), 0)
    checks.push('modal traps/restores focus, closes on Escape and reopens safely')
    await page.locator('a[href="/settings"]').click()
    await page.locator('.settings-page').waitFor()
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'settings-desktop.png') })
    await page.locator('#java-args').fill('-Xmx6G "-Dpath=C:\\Program Files\\Java"')
    await page.locator('#env-vars').fill('TOKEN=a=b== EMPTY= "DISPLAY_NAME=Test pack"')
    await page.waitForFunction(() =>
      window.__nativeSettings.custom_env_vars?.some(([k, v]) => k === 'TOKEN' && v === 'a=b=='),
    )
    assert.deepEqual(await page.evaluate(() => window.__nativeSettings.extra_launch_args), [
      '-Xmx6G',
      '-Dpath=C:\\Program Files\\Java',
    ])
    assert.ok(!(await page.evaluate(() => 'launchArgs' in window.__nativeSettings)))
    checks.push('settings preserve quoted Java paths, empty values and equals')
    await page.evaluate(() => {
      window.__profileListError = true
    })

    await page.locator('a[href="/library"]').click()
    await page.locator('.library-error').waitFor()
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'library-error.png') })
    await page.evaluate(() => {
      window.__profileListError = false
    })
    await page.getByRole('button', { name: 'Повторить', exact: true }).click()
    await page.locator('.no-instance').waitFor()
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'library-empty.png') })
    await page.locator('a[href="/assistant"]').click()
    await page.locator('.assistant-page').waitFor()
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'assistant-desktop.png') })
    await page.getByRole('button', { name: 'Файлы и вылеты', exact: true }).click()
    await page.getByRole('button', { name: 'Открыть файлы', exact: true }).click()
    await page.locator('.ai-file-list .ai-file button').filter({ hasText: 'config' }).click()
    await page
      .locator('.ai-file-list .ai-file button')
      .filter({ hasText: 'config/test.json' })
      .click()
    await page.locator('#ai-file-content').waitFor()
    await page.locator('.ai-file-tools select').selectOption('Other Profile')
    assert.equal(await page.locator('#ai-file-content').count(), 0)
    await page.getByRole('button', { name: 'Открыть файлы', exact: true }).click()
    await page.locator('.ai-file-list .ai-file button').filter({ hasText: 'config' }).click()
    await page
      .locator('.ai-file-list .ai-file button')
      .filter({ hasText: 'config/test.json' })
      .click()
    await page.locator('#ai-file-content').fill('{"renderDistance":12}')
    await page.getByRole('button', { name: 'Сохранить с копией', exact: true }).click()
    await page.locator('.ai-confirm button').last().click()
    await page.waitForFunction(() => window.__aiActions.includes('write_config'))
    const write = await page.evaluate(() =>
      window.__aiCalls.filter((c) => c.tool === 'write_config').at(-1),
    )
    assert.equal(write.args.profileId, 'Other Profile')
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'assistant-files.png') })
    checks.push('profile switches clear the editor; confirmed saves use the bound profile')

    await page.locator('a[href="/minecraft-rust"]').click()
    await page.locator('.minecraft-rust').waitFor()
    await page.waitForTimeout(650) // Wait for the outer route/modal transition, not just DOM visibility.
    await page.screenshot({ animations: 'disabled', path: path.join(out, 'rust-desktop.png') })
    checks.push('settings, empty library, assistant and Rust routes mount')
    assert.deepEqual(errors, [], 'uncaught browser exceptions')
    fs.writeFileSync(
      path.join(out, 'results.json'),
      JSON.stringify({ passed: true, checks, errors }, null, 2),
    )
    console.log(JSON.stringify({ passed: true, checks }, null, 2))
  } finally {
    await browser.close()
  }
})().catch((e) => {
  console.error(e)
  process.exit(1)
})
