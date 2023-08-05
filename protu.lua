json = require("json")
local http = http
local sleep = sleep
require("nav")

local API = "http://192.168.1.92:8080/"

local function cresp(code, output)
	return json.encode({
		code = code,
		out = output,
	})
end

local function cmdcomplete(body)
	local _, err = http.post({
		url = API .. "cmdcomplete",
		headers = {
			["Content-Type"] = "application/json",
		},
		body = body,
	})
end

local function next()
	local resp, err = http.get(API .. "next")
	if err ~= nil then
		return
	end
	--local j = json.decode(resp.readAll())
	--local cmd = j["cmd"]
	return resp.readAll()
end

local cmd, func, rtnval

-- while true
while true do
	cmd = next()
	--print("iter: " .. i .. " | cmd: ".. cmd)
	if cmd == "WAIT" then
		sleep(0.1)
	elseif cmd == "EXIT" then
		cmdcomplete(cresp(0, "Turtle leaving the chat."))
		break
	elseif cmd == nil then
		cmdcomplete(cresp(-1, "Bad request. Unable to execute code."))
		sleep(0.1)
	else
		cmd = "return " .. cmd
		func, _ = load(cmd)
		if not func then
			cmdcomplete(cresp(-2, "Bad function: " .. cmd))
			-- continue
		else
			rtnval = { func() }
			cmdcomplete(cresp(0, rtnval))
		end
	end
end

