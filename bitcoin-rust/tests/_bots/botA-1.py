import json
import time
import sys

def pj(obj):
    print(json.dumps(obj))
    sys.stdout.flush()    

try:
    pj({"SleepMs": 1000})
    time.sleep(1)
    while True:
        pj({"Send": ["MDgCMQDZDExOs97sRTnQLYtgFjDKpDzmO7Uo5HPP62u6MDimXBpZtGxtwa8dhJe5NBIsJjUCAwEAAQ==", "SEND $100   // By Alice"]})
        time.sleep(1)
        pj({"Send": ["MDgCMQDeoEeA8OtGME/SRwp+ASKVOnjlEUHYvQfo0FLp3+fwVi/SztDdJskjzCRasGk06UUCAwEAAQ==", "SEND $200   // By Alice"]})
        time.sleep(1)
        pj({"Send": ["MDgCMQDOpK8YWmcg8ffNF/O7xlBDq/DBdoUnc4yyWrV0y/X3LF+dddjaGksXzGl3tHskpgkCAwEAAQ==", "SEND $300   // By Alice"]})
        time.sleep(1)

except:
    pass

