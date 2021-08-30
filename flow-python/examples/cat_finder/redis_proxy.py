# MegFlow is Licensed under the Apache License, Version 2.0 (the "License")
#
# Copyright (c) 2019-2021 Megvii Inc. All rights reserved.
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT ARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.

#!/usr/bin/env python
# coding=utf-8

import json
import redis
import base64
import numpy as np
from loguru import logger
from pyflow import register

@register(inputs=['inp'], outputs=['out'])
class RedisProxy:
    def __init__(self, name, args):
        logger.info("init redis pool...")
        self.name = name
        self._mode = args['mode']
        self._prefix = args['prefix']
        self._db = dict()

        ip = args['ip']
        port = args['port']
        try:
            self._pool = redis.ConnectionPool(host=ip, port=port, decode_responses=False)
        except:
            logger.error(f'connect redis failed, ip: {ip}, port: {port}')

    # save feature with highest detection score
    def save_feature(self, r, name, items):
        if len(items) == 0:
            return

        score = 0.0
        best_item = None
        for i in range(len(items)):
            if items[i]["score"] > score:
                best_item = items[i]
                score = items[i]["score"]
        
        assert(best_item['feature'] is not None)
        value = base64.b64encode(best_item['feature'].tobytes())
        r.set(f'{self._prefix}{name}', value)
        return value
        
    def search_key(self, r, feature):
        redis_keys = r.keys(self._prefix + '*')
        for key in redis_keys:
            if key not in self._db:
                value_base64 = r.get(key)
                assert(value_base64 is not None)
                try:
                    self._db[key] = np.frombuffer(base64.b64decode(value_base64), dtype=np.float32)
                except:
                    logger.error(f'decode feature failed, key {key}')

        assert(feature is not None)
        if 0 == len(self._db):
            logger.error("feature db empty")
            return {}

        min_dist = float("inf")
        min_key = ''
        for k, v in self._db.items():
            dist = np.linalg.norm(v-feature)
            logger.info(f'key: {k} dist: {dist}')
            if dist < min_dist:
                min_key = k
                min_dist = dist
        min_key = min_key.decode('utf-8')

        name =  min_key.replace(self._prefix, '', 1)
        # send notification
        r.lpush(f'notification.cat_finder', f'{name} leaving the room')
        return {"name": name, "distance": str(min_dist) }

    def exec(self):
        envelope = self.inp.recv()
        if envelope is None:
            return
        image = envelope.msg
        items = image['items']
        assert(type(items) == list)

        r = redis.Redis(connection_pool=self._pool)
        
        if self._mode == 'save':
            self.save_feature(r, image["extra_data"], items)

            results = []
            for item in items:
                result = dict()
                result['bbox'] = item['bbox'].tolist()
                result['score'] = str(item['score'])
                results.append(result)
            # self.out.send(envelope.repack(json.dumps(results)))
            self.out.send(envelope)

        elif self._mode == 'search':
            results = self.search_key(r, image['feature'])
            self.out.send(envelope.repack(json.dumps(results)))

        else:
            logger.error(f'unknown mode: {self._mode}')
            self.out.send(envelope)
            
