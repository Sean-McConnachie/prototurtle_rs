json = require("json")
local http = http
local sleep = sleep

local TURTLE_ID = "/19"

local API = "http://192.168.1.92:8080/"

local function cresp(code, output)
        return json.encode({
                code = code,
                out = output,
        })
end

local function cmdcomplete(body)
        local _, _ = http.post({
                url = API .. "cmdcomplete" .. TURTLE_ID,
                headers = {
                        ["Content-Type"] = "application/json",
                },
                body = body,
        })
end

local function next()
        local resp, err = http.get(API .. "next" .. TURTLE_ID)
        if err ~= nil then
                return
        end
        --local j = json.decode(resp.readAll())
        --local cmd = j["cmd"]
        return resp.readAll()
end

local function register()
        print("Trying to register...")
        while true do
                local resp, err = http.post({
                        url = API .. "register" .. TURTLE_ID,
                        body = ""
                })
                if err == nil then
                        break
                end
                sleep(1)
        end
        print("Registered!")
end

while true do
        register()

        local cmd, func, rtnval
        local i = 0;
        while true do
                cmd = next()
                if cmd == nil then
                        sleep(1)
                        break
                else
                        print("iter: " .. i .. " | cmd: " .. cmd)
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
                        i = i + 1
                end
        end
end
