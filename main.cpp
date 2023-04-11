#include <cstring>
#include <iostream>
#include <fstream>
#include <vector>
#include <string>
#include <unordered_map>
#include <thread>
#include <chrono>
#include <mutex>
using namespace std;
chrono::time_point<chrono::high_resolution_clock> beginTime;
vector<thread> threadHandles;
bool monitorOn = false;


struct Resource {
    int maxAvailableUnits;
    int consumedUnits;
    string name;
};

typedef enum {
    IDLE,
    WAITING,
    RUNNING, } State;

struct Task {
    string name;
    thread::id id;
    State state;
    int waitTime;
    int busyTime;
    int idleTime;
    int runs;
    vector<Resource> taskResources;
};

unordered_map<string, Resource> resources;
unordered_map<string, Task> tasks;

void readFile(string fileName,unordered_map<string, Resource> &resources,unordered_map<string, Task> &tasks) {
    ifstream f(fileName);
    string line;
    vector<string> lineContents;

    while (getline(f, line)) {
        lineContents.clear();
        char *token = strtok(const_cast<char*> (line.c_str())," ");
        while (token)  {
            string strVersion(token);
            lineContents.push_back(strVersion);
            //cout << token << endl;
            // take subsequent tokens
            token = strtok(NULL," ");
        }
        // process the line contents
        if (lineContents[0] == "resources") {
            unordered_map<string, int> temp_resources;
            for (auto it = begin (lineContents) + 1; it != end (lineContents); ++it) {
                char *r = strtok(const_cast<char*> (it->c_str()),":");
                string r_name(r);
                //cout << "THIS IS R1: " << r_name<<endl;
                r = strtok(NULL,":");
                //cout << "THIS IS R2: " << r<<endl;
                Resource resourceParsed = {
                    stoi(r), 
                    0,
                    r_name,
                };
                resources.insert({r_name,resourceParsed });
                //int units = r
                // while (r)  {
                //     string strVersion(r);
                //     lineContents.push_back(strVersion);
                //     //cout << token << endl;
                //     // take subsequent tokens
                //     r = strtok(NULL,":");
                // }
                
                // it->doSomething ();
            }
        }
        else if (lineContents[0] == "task") {
            Task task;
            task.name = lineContents[1];
            task.busyTime = stoi(lineContents[2]);
            task.idleTime = stoi(lineContents[3]);
            task.state = IDLE;
            task.runs = 0;
            // check and gather resources required
            if (lineContents.size() > 4) {
                for (auto it = begin (lineContents) + 4; it != end (lineContents); ++it) {
                    char *r = strtok(const_cast<char*> (it->c_str()),":");
                    string r_name(r);
                    //cout << "THIS IS R1: " << r_name<<endl;
                    r = strtok(NULL,":");
                    //cout << "THIS IS R2: " << r<<endl;
                    Resource resourceParsed = {
                        stoi(r), 
                        0,
                        r_name,
                    };
                    task.taskResources.push_back(resourceParsed);
                }
                tasks.insert({task.name,task});
            }

        }
        // for (auto i: lineContents){
        //     // if (i == "resources") {
        //     //     cout << i << endl;
        //     // }
        //     cout << i << endl;
        // }
        //cout << "END OF LINE "<<endl;

        // cout << "print all elements" <<endl;
        // for (auto itr = tasks.begin(); itr != tasks.end(); itr++) {
        // cout << (itr->first) << " " << (itr->second.name)<< ""<<(itr->second.busyTime) << endl;
        // }
    
  }
}


mutex resourceState;
mutex taskState;
void taskThreadHandler(int iterations,string taskName){
    tasks[taskName].id = this_thread::get_id();

    for (int i = 0; i < iterations; i++) {
        auto startTime = chrono::high_resolution_clock::now();
        if (tasks[taskName].taskResources.size() < 1) {
            tasks[taskName].runs += 1;
            //auto totalTime = chrono::duration_cast<chrono::milliseconds>(chrono::high_resolution_clock::now() - beginTime).count();
            cout << "task: " << tasks[taskName].name << " (tid= " << tasks[taskName].id << ", iter= " << tasks[taskName].runs << ", time= " << 0 << " msec)" << endl;
            continue;
        }
        else {
            //try to get resources
            taskState.lock();
            tasks[taskName].state = WAITING;
            taskState.unlock();

            // the only way we can break out of the loop is if the thread acquires all the resources it needs
            while(1) {
                bool availableResource = true;
                if (resourceState.try_lock()){
                    for (auto res : tasks[taskName].taskResources) {
                        // not enough resources available
                        if ((resources[res.name].maxAvailableUnits - resources[res.name].consumedUnits) < res.maxAvailableUnits) {
                            availableResource = false;
                            break;
                        }
                    }
                    if (!availableResource) {
                        resourceState.unlock();
                        // wait a bit before trying to aquire resource again
                        this_thread::sleep_for(chrono::milliseconds(50));
                    }
                    else{
                        for (auto rec : tasks[taskName].taskResources) {
                            resources[rec.name].consumedUnits += rec.maxAvailableUnits;
                            rec.consumedUnits += rec.maxAvailableUnits;
                        }
                        resourceState.unlock();
                        // end wait time
                        auto endTime = chrono::high_resolution_clock::now();
                        tasks[taskName].waitTime += chrono::duration_cast<chrono::milliseconds>(endTime - startTime).count();
                        break;
                    }
                }
                else {
                    // wait a bit before trying to aquire resource again
                    this_thread::sleep_for(chrono::milliseconds(50));
                }
            }

            //run the task
            taskState.lock();
            tasks[taskName].state = RUNNING;
            taskState.unlock();
            // "run" the task by sleeping for some time
            this_thread::sleep_for(chrono::milliseconds(tasks[taskName].busyTime));

            // release resources
            resourceState.lock();
            for (auto rec : tasks[taskName].taskResources) {
                resources[rec.name].consumedUnits -= rec.maxAvailableUnits;
                rec.consumedUnits -= rec.maxAvailableUnits;
            }
            resourceState.unlock();

            //idle task
            taskState.lock();
            tasks[taskName].state = IDLE;
            taskState.unlock();
            this_thread::sleep_for(chrono::milliseconds(tasks[taskName].idleTime));

            auto totalTime = chrono::duration_cast<chrono::milliseconds>(chrono::high_resolution_clock::now() - beginTime).count();
            tasks[taskName].runs += 1;
            cout << "task: " << tasks[taskName].name << " (tid= " << tasks[taskName].id << ", iter= " << tasks[taskName].runs << ", time= " << totalTime << " msec)" << endl;
        }
    }
}

void monitor() {

}


// a4w23 inputFile monitorTime NITER
int main(int argc, char *argv[]) {
    if (argc == 4) {
        beginTime = chrono::high_resolution_clock::now();
        string tasksFile = argv[1];
        //int monitoring_interval = stoi(argv[2]);
        int nIter = stoi(argv[3]);
        readFile(tasksFile,resources,tasks);
        for (auto task : tasks) {
            // threadHandles.push_back(thread(taskThreadHandler, tasks, task.first, nIter, resources));
            threadHandles.push_back(thread(taskThreadHandler,nIter,task.first));
        }
        for (auto& threadHandle : threadHandles) {
            // wait for thread to terminate
            threadHandle.join();
        }

        cout << "System Resources:"<<endl;
        for (auto rec : resources) {
            cout << rec.first <<": (maxAvail= " << rec.second.maxAvailableUnits<<", held= "<<rec.second.consumedUnits <<")"<<endl;
        }

        cout << "System Tasks:"<<endl;
        for (auto task : tasks) {
            cout<<"-----------------------------------------------------------"<<endl;
            cout << task.first << " (" << task.second.state << ", runTime=" << task.second.busyTime << " msec, idleTime= " << task.second.idleTime << " msec):"<<endl;
            cout << "   (tid= " << task.second.id << ")" <<endl;
            for (auto rec : tasks[task.first].taskResources) {
                cout <<"    "<< rec.name << ": (needed= " << rec.maxAvailableUnits << ", held= " << rec.consumedUnits <<")"<< endl;
            }
            cout << "   (RUN: " << task.second.runs << " times, WAIT: " << task.second.waitTime << " msec)"<<endl;
        }
        cout << "   Running time= " << chrono::duration_cast<chrono::milliseconds>(chrono::high_resolution_clock::now() - beginTime).count() << " msec" << endl;
    }
    else{
        cout << "Error: Incorrect arguments" << endl;
        exit(1);
    }
    
}
