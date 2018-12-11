require("dotenv").config() 
const express = require("express")
const { decorateApp } = require("@awaitjs/express")
const { spawn } = require("child_process")
const Sentry = require('@sentry/node')

Sentry.init({ dsn: process.env.SENTRY_DSN });

const app = decorateApp(express())
app.use(Sentry.Handlers.requestHandler())
app.use(express.json())

app.use((err, req, res, next) => {
  res.setHeader("Access-Control-Allow-Origin", "*")
  next()
})

function runDivvunChecker(tag, text) {
  const checker = spawn("divvun-checker", ["-a", `${tag}.zcheck`])
  
  return new Promise((resolve, reject) => {
    let out = []
    let err = ""

    checker.stdout.on('data', (data) => {
      out.push(JSON.parse(data))
    })
    
    checker.stderr.on('data', (data) => {
      err += data
    })
    
    checker.on('exit', (code) => {
      if (code != 0) {
        reject(new Error(err))
      } else {
        resolve(out)
      }
    })

    checker.stdin.write(text)
    checker.stdin.end()
  })
}

app.postAsync("/grammar/:tag", async (req, res) => {
  const { tag } = req.params
  const { text } = req.body

  res.setHeader('Content-Type', 'application/json')

  if (text == null || typeof text !== "string") {
    res.status(400)
    res.send(JSON.stringify({ error: "`text` field must not be null and must be a string" }))
    return
  }

  try {
    const results = await runDivvunChecker(tag, text)
    res.send(JSON.stringify({ results }))
  } catch (err) {
    if (err.message.startsWith("Archive path not OK")) {
      res.status(400)
      res.send(JSON.stringify({ error: "Language not supported" }))
    } else {
      res.status(500)
      res.send(JSON.stringify({ error: "Internal server error" }))
    }
  }
})

app.use(Sentry.Handlers.errorHandler())

app.use(function onError(err, req, res, next) {
  // The error id is attached to `res.sentry` to be returned
  // and optionally displayed to the user for support.
  res.statusCode = 500
  res.send(JSON.stringify({ error: "Internal server error", id: res.sentry }))
})

const port = process.env.PORT || 8000
const host = process.env.HOST || "127.0.0.1"
app.listen(port, host, () => console.log(`Listening on http://${host}:${port}`))
