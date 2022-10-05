### Terminology and Definitions

This subsection provides a list of terms that are used in the subsequent sections and their meaning, with the purpose of reducing ambiguity and making it easier to correctly understand the requirements.

- Landmark
    - easily detectable & distinguishable
    - unique
- State Estimation
    - process of estimating the state of the robot and its environment
- LiDAR
    - short for "Light Detection And Ranging"
    - a type of distance sensor
- Point cloud
    - a set (cloud) of positions (points)
    - 3D scans are commonly output as point clouds
- Pose
    - a term for the position and orientation of an object
    - usually modeled mathematically in 3D as a 3D position vector along with 3 angles describing the orientation.
    - these angles are commonly labeled yaw, pitch and roll

## Problem Description

LiDart is intended to make accurate [1] 3D scanning available to applications where it is currently cost or complexity prohibitive.

To do this, the end product must be accurate [1], low cost and easy to setup and operate.

allow for accurate [1] 3D scans in a low cost, easy to use package.

### Goal Statements

- Accurate 3D Scanning
    - Ability to product an accurate pointcloud 

- Low Cost Hardware

- Ease of Use

----------------------------

3D scanning is a versatile technology that is used across many industries, but its uses are often limited by high cost and complexity. LiDart aims to build a low cost, simple to use 3D scanning robot. We plan to do this by using easily available low cost LiDAR sensors and consumer grade webcams alongside inexpensive location markers. A software suite will process data obtainted from the robot and provide a user interface. 

LiDart's end product will be a wheel based mobile robot with all required sensors onboard that can be connected to over WiFi.

----------------------------

## Solution Characteristics Specification

The solution that LiDart proposes to the problem specified in section 4.1 is a two wheeled mobile robot that can be remotely driven to take 3D scans of its environment.

The robot will have all required sensors onboard, these sensors will include:
- One or more cameras
- A LiDAR module capable of measuring the distance to objects in a given plane relative to the sensor's orientation and position 
- Two encoders capable of measuring wheel rotations, one for each of the two wheels

----------------------------

# 4 Specific System Description

## 4.1 Assumptions

It is assumed that the environment that LiDart will operate within will have the following properties.

- Not exposed to the elements (eg. Rain)
The robot will not be weather proofed, as this increases cost and complexity. A weather proofed configuration could be designed at a later date by modifiying the current robot.

- Floor should be flat and suitable for driving on with plastic wheels (dry, hard, not slippery. eg. tile, hardwood, concrete)
The robot's simple 2 wheeled drive train is not suited to all terrains. This drive train was chosen to minimize cost and complexity.

- Will remain static while the robot is active
This ensures there are no artifacts in the final stitched 3D scan resulting from changes to the environment between individual 3D scans.

- Prepopulated with landmarks 
This choice was made to simplify the operation of the state estimation algorithm. This should reduce the need for the user to understand the inner workings of the state estimation process.

- No translucent or reflective objects
Translucent and reflective objects can result in poorly behaved LiDAR measurements, and may impact the final stitched 3D scan.

- Is illuminated such that the onboard camera(s) can clearly view the surrounding
The robot relies on cameras for state estimation, as such it must operate in a sufficiently lit environment.

## 4.2 Behaviour Overview

[Figure](Figures/Behaviour%20Overview.pdf)

The figure above (???) shows the high level behaviour of the LiDart system.

Normal operation of the LiDart system will proceed as follows:
1. Robot is powered on
2. User connects to the robot via WiFi
3. User opens the GUI
4. User drives the robot to a desired location, the user can use the live video feed and LiDAR data preview to choose such a location
5. User initiates a scan
6. Steps 4 and 5 are repeated until the user has scanned all desired areas of the environment
7. User downloads the final stitched 3D scan

## 4.3 Functional Decomposition

[Figure](Figures/Functional%20Decomposition.pdf)

The figure above (???) shows the high level data flow throughout the LiDart system. 

The system receives data from the user through the GUI alongside inputs from the various sensors onboard the robot (monitored variables). These inputs include:
- Initiate scan command
- Cancel scan command
- Drive command
- LiDAR readings
- Camera frames
- Wheel encoder readings

The system will use the listed inputs to produce or drive the following outputs (controlled variables):
- Final stitched 3D scan
- Live LiDAR preview (figure ??, look here https://www.slamtec.com/en/Lidar/A1)
- Live video feed of the robot's environment
- Wheel voltages
- Robot status (eg. Fault occured, Ready to scan)

## 4.4 Subsystem Descriptions

Landmark Extraction
    Given a frame from the onboard camera, the landmark extraction subsystem will find all landmarks in the frame and extract information about them.
    This information will include an ID which can be used to correlate subsequent detections of the same landmark. It will also contain some information about the position, and possibly orientation, of the landmark relative to the camera. The exact form of this information will vary based on the choice of landmark type. Two example landmark types are listed below. 

    Colored Spheres
        Uniquely colored spheres could be used as landmarks. They would be easy to detect in a frame, uniquely identifiable based on their color, and would provide information about the direction of the landmark relative to the camera. Given enough landmarks in view these directions could be used to estimate the camera's pose. The size of the sphere could also be used to estimate distance from the camera to the landmark.

        Although colored spheres are a very simple example of a possible landmark, they have several drawbacks. One drawback is that for them to remain unique, each sphere would need to be a distinct color from everything else in the environment, this quickly becomes difficult and would quickly increase the rate of landmark misdetections.

        Another drawback is the sort of position data provided by using colored spheres, you only get direction and distance, no information about orientation. This increases the number of landmarks required to be in view for an accurate estimation of camera pose. 

    AprilTags (https://april.eecs.umich.edu/software/apriltag)
        AprilTags are a 2D barcode system (similar in concept to QR codes) designed at the University of Michigan for easy and robust pose estimation.
        
        Each AprilTag encodes a unique number which can be used to correlate subsequent detections. AprilTags are also robust to misdetection, as their barcode encodes information such that errors can be detected and rejected. Finally, AprilTags provide pose information for each landmark, which greatly reduces the number of landmarks required to be in view for an accurate estimation of camera pose.

Differential Drive Kinematics
    The differential drive kinematics subsystem will provide an estimate of the pose of the robot using readings from the wheel encoders. This estimate will accumulate error over time due to being a purely feed-forward estimator, but it can be used to suplement a more complex state estimator (see section 'State Estimation').

    Figure ???

    The LiDart robot will use a 2 wheel differential drive system, the derivation for the kinematice of such a system is as follows.

    Constants:
    r - wheel radius
    b - the distance between the centers of the wheels

    Variables:
    V - velocity vector of the robot
    VL - velocity vector of the left wheel
    VR - velocity vector of the right wheel
    omega - angular velocity of the robot 
    wL - velocity of the left wheel in the direction that it rotates
    wR - velocity of the right wheel in the direction that it rotates
    Vx - x component of the robot velocity
    Vy - y component of the robot velocity

    Rolling without slipping:
    wL = r*omegaL
    wR = r*omegaR

    Kinematics:
    VR = V + (omega zhat)x(b/2)xhat
    VR = V + (omega)(b/2)yhat
    wR*yhat = Vx*xhat + Vy*yhat + (omega)(b/2)yhat
    
    wR = Vy + (omega)(b/2)  <- y component
    0 = Vx                  <- x component

    Result:
    omegaR = (Vy + omega*b/2) / r
    omegaL = (Vy - omega*b/2) / r

Movement Controller
    The movement controller subsystem will receive commands that the user issues via the GUI and control the drive train as necessary.
    
    While the robot is in scanning mode this subsystem will attempt to keep the robot stationary, and as such will ignore any drive commands. While out of scanning mode this subsystem will control the drive train according to any received drive commands.

    The movement controller will make use of a closed loop velocity controller to control the wheel speeds of the drive train. In this system the velocity of the wheels will be the measured variables, and the voltage given to the wheel motors will be the actuated/controlled variables.

    An example closed loop controller that is suitable for this use case is a PID controller, the discrete time equation for which is as follows.

    K_p - proportional gain
    K_i - integral gain
    K_d - derivative gain
    e_n - error at time n 
    u_n - output of the controller at time n

    u_n = K_p * e_n + K_i * sigma_i e_i + K_d * (e_n - e_n-1)

State Estimation
    The state estimation subsystem will receive data from the landmark extraction and differential drive kinematics subsystems and use that data to estimate the pose of the robot alongside the poses of all the landmarks it has seen.
    
    This state estimation can be done using various different algorithms, one example of such algorithm is the particle filter.

Point Cloud Stitching
    The point cloud stitching subsystem will take the raw LiDAR readings along with the pose estimation from the state estimation subsystem and use that information to add data to the current 3D scan.

    It will do this by first transforming the raw LiDAR readings into the reference frame of the environment, as the raw readings will initially be in the LiDAR sensor's local reference frame. This will be done using the pose estimate provided by the state estimation subsystem.

    It will then take these transformed points and stitch them into the current 3D scan. There are many algorithms for point cloud stitching, two examples of such algorithms are Iterative Closest Point and Moving Least Squares.

----------------------------------
- scope - 1
- context diagram (showing boundaries) - 3

- monitor and controlled variables - 4
- constants - 4
- behvaiour overview - 4
- undesired event handling - 4
- normal operation - 4

- required behvaiour (functional requirements) - 5
- nonfunctional requirements (perf requirements) - 5
- requirements (likely to change) - 6/7
- stakeholders/user identified - 2/3